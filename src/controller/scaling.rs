use super::*;
use paydirt_store::Transaction;

pub(super) fn desired(app: &App) -> u32 {
    if app.scaling.as_option().is_some() {
        app.state.desired
    } else {
        app.replicas
    }
}

pub(super) fn normalize(mut policy: Scaling, max: u32) -> Result<Scaling, ConnectError> {
    if policy.max == 0 {
        policy.max = 8.min(max);
    }
    if policy.target == 0 {
        policy.target = 16;
    }
    if policy.up_ms == 0 {
        policy.up_ms = 1000;
    }
    if policy.down_ms == 0 {
        policy.down_ms = 30000;
    }
    if policy.idle_ms == 0 {
        policy.idle_ms = 60000;
    }
    if policy.waiting == 0 {
        policy.waiting = 64;
    }
    if policy.wait_ms == 0 {
        policy.wait_ms = 120000;
    }
    if policy.min > policy.max
        || policy.max > max
        || policy.target > 10000
        || !(100..=600000).contains(&policy.up_ms)
        || !(1000..=3600000).contains(&policy.down_ms)
        || !(1000..=86400000).contains(&policy.idle_ms)
        || policy.waiting > 4096
        || !(1000..=600000).contains(&policy.wait_ms)
    {
        return Err(ConnectError::invalid_argument(
            "invalid scaling min/max, target, windows or queue limits",
        ));
    }
    Ok(policy)
}

pub(super) fn decide(app: &App, observations: &[Traffic], date: u64, fresh_ms: u64) -> ScaleState {
    let policy = &app.scaling;
    let mut state = (*app.state).clone();
    let elapsed = date.saturating_sub(state.date_sample).min(fresh_ms);
    let mut started = 0u64;
    let mut busy = 0u64;
    let mut active = 0u32;
    let mut waiting = 0u32;
    let mut fresh = true;
    for t in observations
        .iter()
        .filter(|t| t.app.id == app.id && t.revision == app.revision)
    {
        started = started.saturating_add(t.started);
        busy = busy.saturating_add(t.busy_ms);
        if t.closed && t.active == 0 {
            continue;
        }
        fresh &= t.date_seen <= date && date.saturating_sub(t.date_seen) <= fresh_ms;
        active = active.saturating_add(t.active);
        if t.activation && t.placement.as_option().is_none() {
            waiting = waiting.saturating_add(t.active);
        }
    }
    let average = if elapsed > 0 && state.date_sample > 0 {
        busy.saturating_sub(state.busy_ms) as f64 / elapsed as f64
    } else {
        0.0
    };
    let load = average.max(active as f64);
    if state.date_activity == 0 || started > state.started || active > 0 {
        state.date_activity = date;
    }
    let target = (load / policy.target as f64).ceil().min(policy.max as f64) as u32;
    let mut wanted = target.max(policy.min);
    if waiting > 0 {
        wanted = wanted.max(1);
    }
    let idle =
        active == 0 && waiting == 0 && date.saturating_sub(state.date_activity) >= policy.idle_ms;
    if wanted == 0 && !idle {
        wanted = state.desired.min(1).max(policy.min);
    }
    if wanted > state.desired {
        state.low_ms = 0;
        state.high_ms = state.high_ms.saturating_add(elapsed);
        if state.desired == 0 || state.high_ms >= policy.up_ms {
            state.desired = wanted.min(state.desired.saturating_add(1)).min(policy.max);
            state.high_ms = 0;
            state.date_change = date;
        }
    } else if wanted < state.desired && fresh {
        state.high_ms = 0;
        state.low_ms = state.low_ms.saturating_add(elapsed);
        if state.low_ms >= policy.down_ms && (wanted > 0 || idle) {
            state.desired = state.desired.saturating_sub(1).max(wanted);
            state.low_ms = 0;
            state.date_change = date;
        }
    } else {
        state.high_ms = 0;
        state.low_ms = 0;
    }
    state.desired = state.desired.clamp(policy.min, policy.max);
    state.active = active;
    state.waiting = waiting;
    state.fresh = fresh;
    state.load = load;
    state.started = started;
    state.busy_ms = busy;
    state.date_sample = date;
    state
}

pub(super) fn reconcile(
    tx: &mut Transaction<'_>,
    apps: &mut [App],
    traffic: &[Traffic],
    date: u64,
    fresh_ms: u64,
) -> paydirt_store::Result<()> {
    for app in apps.iter_mut().filter(|a| a.scaling.as_option().is_some()) {
        // Every replica sees the same durable sample boundary; a second
        // controller cannot advance a stabilization window twice for one tick.
        if date.saturating_sub(app.state.date_sample) < 100 {
            continue;
        }
        let state = decide(app, traffic, date, fresh_ms);
        *app = decode(call(
            tx,
            "App",
            "Patch",
            json!({"ref":{"id":encode(&app.id)},"state":state}),
        )?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(desired: u32) -> App {
        App {
            id: vec![1],
            revision: 2,
            scaling: Some(
                normalize(
                    Scaling {
                        min: 0,
                        max: 4,
                        target: 2,
                        up_ms: 1000,
                        down_ms: 1000,
                        idle_ms: 1000,
                        ..Default::default()
                    },
                    4,
                )
                .unwrap(),
            )
            .into(),
            state: Some(ScaleState {
                desired,
                date_sample: 1000,
                date_activity: 1000,
                ..Default::default()
            })
            .into(),
            ..Default::default()
        }
    }
    fn observed(active: u32, date: u64) -> Traffic {
        Traffic {
            app: Some(App {
                id: vec![1],
                ..Default::default()
            })
            .into(),
            revision: 2,
            active,
            started: active as u64,
            date_seen: date,
            ..Default::default()
        }
    }
    #[test]
    fn sustained_load_grows_one_at_a_time_and_stale_reports_never_authorize_downscale() {
        let mut a = app(1);
        let t = observed(100, 1500);
        a.state = Some(decide(&a, std::slice::from_ref(&t), 1500, 1000)).into();
        assert_eq!(a.state.desired, 1);
        a.state = Some(decide(&a, &[observed(100, 2000)], 2000, 1000)).into();
        assert_eq!(a.state.desired, 2);
        let stale = observed(0, 1000);
        a.state = Some(decide(&a, &[stale], 10000, 1000)).into();
        assert_eq!(a.state.desired, 2);
        assert!(!a.state.fresh);
        a.state = Some(decide(&a, &[], 11000, 1000)).into();
        assert_eq!(a.state.desired, 1);
        a.state = Some(decide(&a, &[], 12000, 1000)).into();
        assert_eq!(a.state.desired, 0);
    }
    #[test]
    fn closed_gate_with_an_open_stream_remains_active_and_old_revisions_do_not_wake() {
        let mut a = app(1);
        let mut t = observed(1, 5000);
        t.closed = true;
        let state = decide(&a, &[t.clone()], 5000, 1000);
        assert_eq!(state.active, 1);
        assert_eq!(state.desired, 1);
        a.state.get_or_insert_default().desired = 0;
        t.revision = 1;
        t.busy_ms = 100000;
        let state = decide(&a, &[t], 5000, 1000);
        assert_eq!(state.desired, 0);
        assert_eq!(state.active, 0);
    }
}
