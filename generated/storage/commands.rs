async fn dispatch(
    client: &HttpClient,
    config: &ClientConfig,
    entity: &str,
    rpc: &str,
    request: Value,
    mut emit: impl FnMut(Value) -> Result<()>,
) -> Result<()> {
    use connectrpc::client::{call_unary, call_server_stream};
    match (entity, rpc) {
        ("wafl.Access", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AccessGetRequest,
                <proto::r#wafl::r#Access as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AccessService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Access", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AccessListRequest,
                <proto::r#wafl::r#AccessListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.AccessService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "Add") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppAddRequest,
                <proto::r#wafl::r#App as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "Add",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "Apply") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppApplyRequest,
                <proto::r#wafl::r#App as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "Apply",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "Erase") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppRef,
                <proto::r#wafl::r#AppEraseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "Erase",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppGetRequest,
                <proto::r#wafl::r#App as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppListRequest,
                <proto::r#wafl::r#AppListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.App", "Patch") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AppPatchRequest,
                <proto::r#wafl::r#App as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AppService",
                    "Patch",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "Add") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditAddRequest,
                <proto::r#wafl::r#Audit as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "Add",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "Apply") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditApplyRequest,
                <proto::r#wafl::r#Audit as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "Apply",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "Erase") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditRef,
                <proto::r#wafl::r#AuditEraseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "Erase",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditGetRequest,
                <proto::r#wafl::r#Audit as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditListRequest,
                <proto::r#wafl::r#AuditListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Audit", "Patch") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#AuditPatchRequest,
                <proto::r#wafl::r#Audit as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.AuditService",
                    "Patch",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Controller", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#ControllerGetRequest,
                <proto::r#wafl::r#Controller as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.ControllerService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Add") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderAddRequest,
                <proto::r#wafl::r#Holder as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Add",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Apply") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderApplyRequest,
                <proto::r#wafl::r#Holder as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Apply",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Erase") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderRef,
                <proto::r#wafl::r#HolderEraseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Erase",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderGetRequest,
                <proto::r#wafl::r#Holder as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderListRequest,
                <proto::r#wafl::r#HolderListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Patch") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#HolderPatchRequest,
                <proto::r#wafl::r#Holder as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Patch",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Holder", "Watch") => {
            let mut stream = call_server_stream::<
                _,
                proto::r#wafl::r#HolderWatchRequest,
                <proto::r#wafl::r#HolderWatchResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.HolderService",
                    "Watch",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            while let Some(msg) = stream
                .message::<proto::r#wafl::r#HolderWatchResponse>()
                .await?
            {
                emit(serde_json::to_value(msg.to_owned_message())?)?;
            }
        }
        ("wafl.Instance", "Add") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceAddRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Add",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Drain") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceDrainRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Drain",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Erase") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceRef,
                <proto::r#wafl::r#InstanceEraseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Erase",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceGetRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceListRequest,
                <proto::r#wafl::r#InstanceListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Release") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceReleaseRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Release",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Start") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceStartRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Start",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Stop") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#InstanceStopRequest,
                <proto::r#wafl::r#Instance as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Stop",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Instance", "Tail") => {
            let mut stream = call_server_stream::<
                _,
                proto::r#wafl::r#InstanceTailRequest,
                <proto::r#wafl::r#Log as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.InstanceService",
                    "Tail",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            while let Some(msg) = stream.message::<proto::r#wafl::r#Log>().await? {
                emit(serde_json::to_value(msg.to_owned_message())?)?;
            }
        }
        ("wafl.Placement", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#PlacementGetRequest,
                <proto::r#wafl::r#Placement as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.PlacementService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Placement", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#PlacementListRequest,
                <proto::r#wafl::r#PlacementListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.PlacementService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Placement", "Release") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#PlacementReleaseRequest,
                <proto::r#wafl::r#Placement as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.PlacementService",
                    "Release",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Reservation", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#ReservationGetRequest,
                <proto::r#wafl::r#Reservation as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.ReservationService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Reservation", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#ReservationListRequest,
                <proto::r#wafl::r#ReservationListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.ReservationService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Reservation", "Release") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#ReservationReleaseRequest,
                <proto::r#wafl::r#ReservationReleaseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.ReservationService",
                    "Release",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Runner", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#RunnerGetRequest,
                <proto::r#wafl::r#Runner as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.RunnerService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Runner", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#RunnerListRequest,
                <proto::r#wafl::r#RunnerListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.RunnerService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "Add") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantAddRequest,
                <proto::r#wafl::r#Tenant as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "Add",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "Apply") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantApplyRequest,
                <proto::r#wafl::r#Tenant as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "Apply",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "Erase") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantRef,
                <proto::r#wafl::r#TenantEraseResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "Erase",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "Get") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantGetRequest,
                <proto::r#wafl::r#Tenant as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "Get",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "List") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantListRequest,
                <proto::r#wafl::r#TenantListResponse as buffa::HasMessageView>::View<
                    'static,
                >,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "List",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        ("wafl.Tenant", "Patch") => {
            let res = call_unary::<
                _,
                proto::r#wafl::r#TenantPatchRequest,
                <proto::r#wafl::r#Tenant as buffa::HasMessageView>::View<'static>,
            >(
                    client,
                    config,
                    "wafl.TenantService",
                    "Patch",
                    serde_json::from_value(request)?,
                    Default::default(),
                )
                .await?;
            emit(serde_json::to_value(res.into_owned())?)?;
        }
        _ => return Err("unsupported command".into()),
    }
    Ok(())
}
