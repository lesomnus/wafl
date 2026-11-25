use super::{lifecycle::call, *};
use serde_json::json;
use wafl_runner::{config::Direct, proto::wafl::Reservation};

#[tokio::test]
async fn reservation_generated_queries_apply_selection_filters_and_pool_bound_cursors() {
    let root = tempfile::tempdir().unwrap();
    let configuration = Config {
        directory: Some(root.path().join("cache")),
        direct: Some(Direct {
            directory: root.path().join("endpoints"),
            listen: "0.0.0.0".parse().unwrap(),
            advertise: "127.0.0.1".parse().unwrap(),
            first: 21000,
            last: 21002,
        }),
        ..config()
    };
    let runner = Runner::new(&configuration).await.unwrap();
    // The read layer must expose occupied journal slots even without Instance
    // objects. A live session proof is created by Runner::new itself.
    for port in 21000..=21002 {
        std::fs::write(
            root.path()
                .join(format!("endpoints/v1/127.0.0.1/{port}.json")),
            serde_json::to_vec(&json!({
                "version":2, "token":uuid::Uuid::new_v4().to_string(),
                "session":paydirt::format_id(&runner.session()).unwrap(),
                "instance":paydirt::format_id(&paydirt::mint(7)).unwrap(), "generation":1
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let query = "/wafl.ReservationService/List";
    let (status, all) = call(&runner, query, json!({}), &[]).await;
    assert_eq!(status, 200, "{all}");
    assert_eq!(all["items"].as_array().unwrap().len(), 3);
    let first: Reservation = serde_json::from_value(all["items"][0].clone()).unwrap();
    assert!(paydirt::has_domain(&first.id, 9));
    let reference = json!({"id":STANDARD.encode(&first.id)});
    let selected = call(
        &runner,
        "/wafl.ReservationService/Get",
        json!({"ref":reference,"select":{"address":true,"status":true}}),
        &[],
    )
    .await;
    assert_eq!(selected.0, 200);
    assert_eq!(selected.1["address"], first.address);
    assert_eq!(selected.1["id"], all["items"][0]["id"]);
    assert_eq!(selected.1["status"], "RESERVATION_STATUS_ACTIVE");
    assert!(selected.1["token"].is_null());
    let first_page = call(&runner, query, json!({"size":1}), &[]).await.1;
    let next = first_page["next"].as_str().unwrap();
    assert_eq!(first_page["items"][0], all["items"][0]);
    // Cursor remains meaningful when its slot is no longer occupied.
    let port = first.address.parse::<SocketAddr>().unwrap().port();
    std::fs::remove_file(
        root.path()
            .join(format!("endpoints/v1/127.0.0.1/{port}.json")),
    )
    .unwrap();
    let rest = call(&runner, query, json!({"after":next}), &[]).await.1;
    assert_eq!(rest["items"], json!(all["items"].as_array().unwrap()[1..]));
    assert!(rest["next"].is_null());
    let reverse = call(&runner, query, json!({"reverse":true}), &[]).await.1;
    assert_eq!(reverse["items"][0], all["items"][2]);
    let filter = json!({"ref":{"address":all["items"][1]["address"]}});
    let filtered = call(&runner, query, json!({"filters":[filter]}), &[])
        .await
        .1;
    assert_eq!(filtered["items"], json!([all["items"][1]]));
    let address: SocketAddr = all["items"][1]["address"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let equivalent = call(
        &runner,
        query,
        json!({"filters":[{"ref":{"address":format!("{}:0{}", address.ip(), address.port())}}]}),
        &[],
    )
    .await;
    assert_eq!(equivalent.0, 200);
    assert_eq!(equivalent.1["items"], filtered["items"]);
    for body in [
        json!({"after":next,"reverse":true}),
        json!({"after":next,"filters":[filter]}),
        json!({"filters":[{}]}),
        json!({"filters":vec![filter;65]}),
        json!({"filters":[{"ref":{"address":"127.0.0.2:21000"}}]}),
        json!({"after":"invalid"}),
    ] {
        assert_eq!(
            call(&runner, query, body, &[]).await.1["code"],
            "invalid_argument"
        );
    }
    let mut other_config = configuration.clone();
    other_config.direct.as_mut().unwrap().last = 21003;
    let other = Runner::new(&other_config).await.unwrap();
    assert_eq!(
        call(&other, query, json!({"after":next}), &[]).await.1["code"],
        "invalid_argument"
    );

    let release = "/wafl.ReservationService/Release";
    let current = &all["items"][1];
    let request = json!({"ref":{"id":current["id"]},"token":current["token"],"withdrawn":true});
    assert_eq!(
        call(&runner, release, request.clone(), &[]).await.1["code"],
        "failed_precondition"
    );
    let mut no_ack = request.clone();
    no_ack["withdrawn"] = false.into();
    assert_eq!(
        call(&runner, release, no_ack, &[]).await.1["code"],
        "invalid_argument"
    );
    let mut wrong_token = request.clone();
    wrong_token["token"] = uuid::Uuid::new_v4().to_string().into();
    assert_eq!(
        call(&runner, release, wrong_token, &[]).await.1["code"],
        "aborted"
    );
    assert_eq!(
        call(
            &runner,
            release,
            request.clone(),
            &[(
                "wafl-session",
                paydirt::format_id(&other.session()).unwrap()
            )]
        )
        .await
        .1["code"],
        "failed_precondition"
    );
    assert_eq!(
        call(
            &runner,
            release,
            request,
            &[
                (
                    "wafl-session",
                    paydirt::format_id(&runner.session()).unwrap()
                ),
                ("wafl-revision", "1".into())
            ]
        )
        .await
        .1["code"],
        "invalid_argument"
    );
    let absent = call(
        &runner,
        release,
        json!({"ref":reference,"token":first.token,"withdrawn":true}),
        &[],
    )
    .await;
    assert_eq!(absent.0, 200);
    assert!(!absent.1["released"].as_bool().unwrap_or(false));
    runner.shutdown().await;
    other.shutdown().await;

    other_config.direct = None;
    let proxy = Runner::new(&other_config).await.unwrap();
    assert_eq!(
        call(&proxy, query, json!({}), &[]).await.1["code"],
        "failed_precondition"
    );
    assert!(
        !proxy
            .status()
            .capabilities
            .iter()
            .any(|c| c == "reservation-recovery-v1")
    );
    proxy.shutdown().await;
}
