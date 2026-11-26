use super::*;
use wafl_runner::{config::RESERVATION_DOMAIN, services::wafl::ReservationServiceClient};

#[derive(Subcommand)]
pub(super) enum Command {
    Get(Target),
    List,
    /// Reclaim a dead owner's reservation after all gateway references are gone.
    Release(Release),
}
#[derive(Args)]
pub(super) struct Target {
    #[arg(value_name = "ID|IP:PORT")]
    reference: String,
}
#[derive(Args)]
pub(super) struct Release {
    #[arg(value_name = "ID|IP:PORT")]
    reference: String,
    /// Allocation token captured before withdrawing routes. Never auto-refresh it.
    #[arg(long)]
    token: String,
    /// Confirm all routes, pools and connections for this allocation are drained.
    #[arg(long, required = true)]
    withdrawn: bool,
}

fn reference(value: &str) -> Result<ReservationRef> {
    let key = if let Ok(address) = value.parse::<std::net::SocketAddrV4>() {
        reservation_ref::Key::Address(address.to_string())
    } else {
        let id = paydirt::parse_id(value)
            .filter(|id| paydirt::has_domain(id, RESERVATION_DOMAIN))
            .context("expected a domain-9 Reservation UUID or IPv4:port")?;
        reservation_ref::Key::Id(id.to_vec())
    };
    Ok(ReservationRef {
        key: Some(key),
        ..Default::default()
    })
}
fn print(value: &Reservation, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string(value)?);
    } else {
        let status = [
            (ReservationStatus::Active, "active"),
            (ReservationStatus::Orphan, "orphan"),
            (ReservationStatus::Unknown, "unknown"),
            (ReservationStatus::Corrupt, "corrupt"),
        ]
        .into_iter()
        .find(|(status, _)| value.status == *status)
        .map(|(_, name)| name)
        .unwrap_or("unspecified");
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            paydirt::format_id(&value.id).context("invalid reservation ID in response")?,
            value.address,
            status,
            value.token,
            paydirt::format_id(&value.session).unwrap_or_default(),
            paydirt::format_id(&value.instance).unwrap_or_default(),
            value.generation,
            value.error
        );
    }
    Ok(())
}
pub(super) async fn run(action: Command, config: ClientConfig, json: bool) -> Result<()> {
    let client = ReservationServiceClient::new(HttpClient::plaintext(), config);
    match action {
        Command::Get(args) => print(
            &client
                .get(ReservationGetRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    ..Default::default()
                })
                .await
                .map(owned)?,
            json,
        )?,
        Command::List => {
            let mut request = ReservationListRequest::default();
            loop {
                let page = client.list(request.clone()).await.map(owned)?;
                for value in &page.items {
                    print(value, json)?;
                }
                if page.next.is_empty() {
                    break;
                }
                request.after = page.next;
            }
        }
        Command::Release(args) => {
            let result = client
                .release(ReservationReleaseRequest {
                    r#ref: Some(reference(&args.reference)?).into(),
                    token: args.token,
                    withdrawn: args.withdrawn,
                    ..Default::default()
                })
                .await
                .map(owned)?;
            if json {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("{}", result.released);
            }
        }
    }
    Ok(())
}
