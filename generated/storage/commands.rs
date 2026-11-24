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
