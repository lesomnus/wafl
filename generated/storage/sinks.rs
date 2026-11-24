#[path = "../server.rs"]
pub mod server;
#[path = "intercept.rs"]
pub mod intercept;
use crate::transport::Database;
use ::connectrpc::{RequestContext, Response, Router, ServiceRequest, ServiceResult};
impl<
    N: crate::services::r#payday::GrantService,
> crate::services::r#payday::GrantServiceLayer<N> for crate::storage::intercept::Layer {
    fn watch(
        &self,
        next: &N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::payday::GrantWatchRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::payday::GrantWatchResponse,
                > + Send + use<N>,
            >,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/payday.GrantService/Watch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.watch(call.ctx.clone(), request).await;
            call.stream::<crate::proto::r#payday::r#GrantWatchResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::payday::GrantListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::payday::GrantListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/payday.GrantService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#payday::r#GrantListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#payday::ResourceService,
> crate::services::r#payday::ResourceServiceLayer<N>
for crate::storage::intercept::Layer {
    fn watch(
        &self,
        next: &N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::payday::WatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::payday::WatchEvent,
                > + Send + use<N>,
            >,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/payday.ResourceService/Watch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.watch(call.ctx.clone(), request).await;
            call.stream::<crate::proto::r#payday::r#WatchEvent, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#payday::TokenService,
> crate::services::r#payday::TokenServiceLayer<N> for crate::storage::intercept::Layer {
    fn introspect<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::payday::TokenIntrospectRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::payday::TokenIntrospectResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/payday.TokenService/Introspect", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.introspect(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#payday::r#TokenIntrospectResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::TenantService,
> crate::services::r#wafl::TenantServiceLayer<N> for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Tenant> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Tenant, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Tenant> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Tenant, _>(reply)
        }
    }
    fn patch<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Tenant> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/Patch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.patch(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Tenant, _>(reply)
        }
    }
    fn apply<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Tenant> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/Apply", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.apply(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Tenant, _>(reply)
        }
    }
    fn erase<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::TenantEraseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/Erase", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.erase(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#TenantEraseResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::TenantListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TenantService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#TenantListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::InstanceService,
> crate::services::r#wafl::InstanceServiceLayer<N> for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn erase<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceEraseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Erase", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.erase(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#InstanceEraseResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceListRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#InstanceListResponse, _>(reply)
        }
    }
    fn start<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceStartRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Start", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.start(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn stop<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceStopRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Stop", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.stop(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::AccessService,
> crate::services::r#wafl::AccessServiceLayer<N> for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AccessGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Access> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AccessService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Access, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AccessListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AccessListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AccessService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AccessListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::ArchivedService,
> crate::services::r#wafl::ArchivedServiceLayer<N> for crate::storage::intercept::Layer {}
impl<
    N: crate::services::r#wafl::AuditService,
> crate::services::r#wafl::AuditServiceLayer<N> for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Audit> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Audit, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Audit> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Audit, _>(reply)
        }
    }
    fn patch<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Audit> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/Patch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.patch(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Audit, _>(reply)
        }
    }
    fn apply<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Audit> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/Apply", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.apply(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Audit, _>(reply)
        }
    }
    fn erase<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AuditEraseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/Erase", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.erase(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AuditEraseResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AuditListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AuditService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AuditListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::HolderService,
> crate::services::r#wafl::HolderServiceLayer<N> for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Holder> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Holder, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Holder> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Holder, _>(reply)
        }
    }
    fn patch<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Holder> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Patch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.patch(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Holder, _>(reply)
        }
    }
    fn apply<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Holder> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Apply", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.apply(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Holder, _>(reply)
        }
    }
    fn erase<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::HolderEraseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Erase", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.erase(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#HolderEraseResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::HolderListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#HolderListResponse, _>(reply)
        }
    }
    fn watch(
        &self,
        next: &N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderWatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderWatchResponse,
                > + Send + use<N>,
            >,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.HolderService/Watch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.watch(call.ctx.clone(), request).await;
            call.stream::<crate::proto::r#wafl::r#HolderWatchResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::OutboxService,
> crate::services::r#wafl::OutboxServiceLayer<N> for crate::storage::intercept::Layer {}
impl<
    N: crate::services::r#wafl::OwedService,
> crate::services::r#wafl::OwedServiceLayer<N> for crate::storage::intercept::Layer {}
impl<
    N: crate::services::r#wafl::RowSealService,
> crate::services::r#wafl::RowSealServiceLayer<N> for crate::storage::intercept::Layer {}
mod sink_wafl_access {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::AccessService,
    > crate::services::r#wafl::AccessServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AccessGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Access> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Access", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AccessListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AccessListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Access", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::AccessService,
    > crate::services::r#wafl::AccessServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AccessGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Access,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.AccessGetRequest",
                    "wafl.Access",
                    "/wafl.AccessService/Get",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Access",
                        "Get",
                        self.public.contains(&"/wafl.AccessService/Get"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.get(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Access,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AccessListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AccessListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.AccessListRequest",
                    "wafl.AccessListResponse",
                    "/wafl.AccessService/List",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Access",
                        "List",
                        self.public.contains(&"/wafl.AccessService/List"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.list(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#AccessListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::AccessService,
    > crate::services::r#wafl::AccessServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AccessGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Access,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Access",
                        "Get",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.get(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AccessListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AccessListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Access",
                        "List",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.list(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_audit {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::AuditService,
    > crate::services::r#wafl::AuditServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AuditGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Audit> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.Audit", "Get", &request.to_owned_message()).await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AuditListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AuditListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Audit", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::AuditService,
    > crate::services::r#wafl::AuditServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Audit,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::AuditService::add(
                &crate::services::r#wafl::AuditServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Audit,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.AuditGetRequest",
                    "wafl.Audit",
                    "/wafl.AuditService/Get",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Audit",
                        "Get",
                        self.public.contains(&"/wafl.AuditService/Get"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.get(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<crate::proto::r#wafl::r#Audit, _>(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn patch<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Audit,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::AuditService::patch(
                &crate::services::r#wafl::AuditServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn apply<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Audit,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::AuditService::apply(
                &crate::services::r#wafl::AuditServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AuditEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::AuditService::erase(
                &crate::services::r#wafl::AuditServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AuditListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.AuditListRequest",
                    "wafl.AuditListResponse",
                    "/wafl.AuditService/List",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Audit",
                        "List",
                        self.public.contains(&"/wafl.AuditService/List"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.list(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#AuditListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::AuditService,
    > crate::services::r#wafl::AuditServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Audit,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Audit",
                        "Get",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.get(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AuditListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AuditListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Audit",
                        "List",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.list(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_holder {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::HolderService,
    > crate::services::r#wafl::HolderServiceLayer<N> for Sink {
        async fn add(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderAddRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Holder> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "Add", &request.to_owned_message())
                    .await?,
            )
        }
        async fn apply(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderApplyRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Holder> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "Apply", &request.to_owned_message())
                    .await?,
            )
        }
        async fn erase(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderRef>,
        ) -> ServiceResult<crate::proto::r#wafl::r#HolderEraseResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "Erase", &request.to_owned_message())
                    .await?,
            )
        }
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Holder> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#HolderListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "List", &request.to_owned_message())
                    .await?,
            )
        }
        async fn patch(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderPatchRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Holder> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Holder", "Patch", &request.to_owned_message())
                    .await?,
            )
        }
        async fn watch(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#HolderWatchRequest>,
        ) -> ServiceResult<
            ::connectrpc::ServiceStream<crate::proto::r#wafl::r#HolderWatchResponse>,
        > {
            self.0.watch(ctx, "wafl.Holder", &request.to_owned_message()).await
        }
    }
    impl<
        N: crate::services::r#wafl::HolderService,
    > crate::services::r#wafl::HolderServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderAddRequest",
                    "wafl.Holder",
                    "/wafl.HolderService/Add",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Add",
                        self.public.contains(&"/wafl.HolderService/Add"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.add(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Holder,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderGetRequest",
                    "wafl.Holder",
                    "/wafl.HolderService/Get",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Get",
                        self.public.contains(&"/wafl.HolderService/Get"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.get(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Holder,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn patch<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderPatchRequest",
                    "wafl.Holder",
                    "/wafl.HolderService/Patch",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Patch",
                        self.public.contains(&"/wafl.HolderService/Patch"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.patch(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Holder,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn apply<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderApplyRequest",
                    "wafl.Holder",
                    "/wafl.HolderService/Apply",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Apply",
                        self.public.contains(&"/wafl.HolderService/Apply"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.apply(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Holder,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderRef",
                    "wafl.HolderEraseResponse",
                    "/wafl.HolderService/Erase",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Erase",
                        self.public.contains(&"/wafl.HolderService/Erase"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.erase(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#HolderEraseResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderListRequest",
                    "wafl.HolderListResponse",
                    "/wafl.HolderService/List",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "List",
                        self.public.contains(&"/wafl.HolderService/List"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.list(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#HolderListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn watch(
            &self,
            next: &N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderWatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                ::connectrpc::ServiceStream<
                    impl ::connectrpc::Encodable<
                        crate::proto::wafl::HolderWatchResponse,
                    > + Send + use<N>,
                >,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.HolderWatchRequest",
                    "wafl.HolderWatchResponse",
                    "/wafl.HolderService/Watch",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Holder",
                        "Watch",
                        self.public.contains(&"/wafl.HolderService/Watch"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.watch(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                Ok(
                    response
                        .map_body(|body| {
                            attempt
                                .stream::<
                                    crate::proto::r#wafl::r#HolderWatchResponse,
                                    _,
                                >(self.db.clone(), Database::enforce_presence(&ctx, body))
                        }),
                )
            }
        }
    }
    impl<
        N: crate::services::r#wafl::HolderService,
    > crate::services::r#wafl::HolderServiceLayer<N> for Audit {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "Add",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.add(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "Get",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.get(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn patch<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "Patch",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.patch(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn apply<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Holder,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "Apply",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.apply(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "Erase",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.erase(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::HolderListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Holder",
                        "List",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.list(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_instance {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::InstanceService,
    > crate::services::r#wafl::InstanceServiceLayer<N> for Sink {
        async fn add(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#InstanceAddRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Instance> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Instance", "Add", &request.to_owned_message())
                    .await?,
            )
        }
        async fn erase(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#InstanceRef>,
        ) -> ServiceResult<crate::proto::r#wafl::r#InstanceEraseResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Instance", "Erase", &request.to_owned_message())
                    .await?,
            )
        }
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#InstanceGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Instance> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Instance", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#InstanceListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#InstanceListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Instance", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::InstanceService,
    > crate::services::r#wafl::InstanceServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceAddRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Add",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "Add",
                        self.public.contains(&"/wafl.InstanceService/Add"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.add(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Instance,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceGetRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Get",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "Get",
                        self.public.contains(&"/wafl.InstanceService/Get"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.get(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Instance,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceRef",
                    "wafl.InstanceEraseResponse",
                    "/wafl.InstanceService/Erase",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "Erase",
                        self.public.contains(&"/wafl.InstanceService/Erase"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.erase(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#InstanceEraseResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceListRequest",
                    "wafl.InstanceListResponse",
                    "/wafl.InstanceService/List",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "List",
                        self.public.contains(&"/wafl.InstanceService/List"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.list(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#InstanceListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn start<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceStartRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceStartRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Start",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "Start",
                        self.public.contains(&"/wafl.InstanceService/Start"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.start(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Instance,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn stop<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceStopRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.InstanceStopRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Stop",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Instance",
                        "Stop",
                        self.public.contains(&"/wafl.InstanceService/Stop"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.stop(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Instance,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::InstanceService,
    > crate::services::r#wafl::InstanceServiceLayer<N> for Audit {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "Add",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.add(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "Get",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.get(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "Erase",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.erase(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "List",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.list(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn start<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceStartRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "Start",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.start(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn stop<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceStopRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Instance,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Instance",
                        "Stop",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.stop(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_tenant {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::TenantService,
    > crate::services::r#wafl::TenantServiceLayer<N> for Sink {
        async fn apply(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TenantApplyRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Tenant> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Tenant", "Apply", &request.to_owned_message())
                    .await?,
            )
        }
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TenantGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Tenant> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Tenant", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TenantListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#TenantListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Tenant", "List", &request.to_owned_message())
                    .await?,
            )
        }
        async fn patch(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TenantPatchRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Tenant> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Tenant", "Patch", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::TenantService,
    > crate::services::r#wafl::TenantServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::TenantService::add(
                &crate::services::r#wafl::TenantServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.TenantGetRequest",
                    "wafl.Tenant",
                    "/wafl.TenantService/Get",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Tenant",
                        "Get",
                        self.public.contains(&"/wafl.TenantService/Get"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.get(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Tenant,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn patch<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.TenantPatchRequest",
                    "wafl.Tenant",
                    "/wafl.TenantService/Patch",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Tenant",
                        "Patch",
                        self.public.contains(&"/wafl.TenantService/Patch"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.patch(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Tenant,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn apply<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.TenantApplyRequest",
                    "wafl.Tenant",
                    "/wafl.TenantService/Apply",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Tenant",
                        "Apply",
                        self.public.contains(&"/wafl.TenantService/Apply"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.apply(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Tenant,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::TenantEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            let _ = next;
            crate::services::r#wafl::TenantService::erase(
                &crate::services::r#wafl::TenantServiceUnimplemented,
                ctx,
                request,
            )
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::TenantListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let ctx = Database::presence_request(
                    ctx,
                    || request.to_owned_message(),
                )?;
                let mut ctx = ctx;
                let attempt = crate::transport::access::Attempt::start(
                    &mut ctx,
                    "wafl.TenantListRequest",
                    "wafl.TenantListResponse",
                    "/wafl.TenantService/List",
                    false,
                    || {
                        serde_json::to_value(request.to_owned_message())
                            .map_err(|_| connectrpc::ConnectError::invalid_argument(
                                "invalid access request",
                            ))
                    },
                )?;
                let ctx = match self
                    .db
                    .authorize(
                        ctx,
                        "wafl.Tenant",
                        "List",
                        self.public.contains(&"/wafl.TenantService/List"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.list(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#TenantListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::TenantService,
    > crate::services::r#wafl::TenantServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Tenant",
                        "Get",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.get(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn patch<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Tenant",
                        "Patch",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.patch(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn apply<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Tenant,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Tenant",
                        "Apply",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.apply(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn list<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TenantListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::TenantListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Tenant",
                        "List",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.list(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
#[derive(Clone)]
pub struct Gate {
    db: Database,
    public: &'static [&'static str],
}
impl Gate {
    pub fn new(db: Database, public: &'static [&'static str]) -> Self {
        Self { db, public }
    }
}
#[derive(Clone)]
pub struct Audit(pub Database);
#[derive(Clone)]
pub struct Sink(pub Database);
#[derive(Clone)]
pub struct Next(pub Database);
impl Next {
    pub fn wafl_access(&self) -> impl crate::services::r#wafl::AccessService + use<> {
        crate::services::r#wafl::AccessServiceOverlay {
            next: crate::services::r#wafl::AccessServiceUnimplemented,
            layer: sink_wafl_access::Sink(self.0.clone()),
        }
    }
    pub fn wafl_audit(&self) -> impl crate::services::r#wafl::AuditService + use<> {
        crate::services::r#wafl::AuditServiceOverlay {
            next: crate::services::r#wafl::AuditServiceUnimplemented,
            layer: sink_wafl_audit::Sink(self.0.clone()),
        }
    }
    pub fn wafl_holder(&self) -> impl crate::services::r#wafl::HolderService + use<> {
        crate::services::r#wafl::HolderServiceOverlay {
            next: crate::services::r#wafl::HolderServiceUnimplemented,
            layer: sink_wafl_holder::Sink(self.0.clone()),
        }
    }
    pub fn wafl_instance(
        &self,
    ) -> impl crate::services::r#wafl::InstanceService + use<> {
        crate::services::r#wafl::InstanceServiceOverlay {
            next: crate::services::r#wafl::InstanceServiceUnimplemented,
            layer: sink_wafl_instance::Sink(self.0.clone()),
        }
    }
    pub fn wafl_tenant(&self) -> impl crate::services::r#wafl::TenantService + use<> {
        crate::services::r#wafl::TenantServiceOverlay {
            next: crate::services::r#wafl::TenantServiceUnimplemented,
            layer: sink_wafl_tenant::Sink(self.0.clone()),
        }
    }
}
#[allow(async_fn_in_trait)]
pub trait AccessCalls: crate::services::r#wafl::AccessService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AccessGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Access, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AccessGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Access::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#AccessListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Access, ::connectrpc::ConnectError>,
    > + 'a {
        ::paydirt::page::all(
            request,
            |q| &mut q.after,
            move |q| async move {
                let page = self.list_owned(ctx, q).await?;
                Ok((page.items, page.next))
            },
            || ::connectrpc::ConnectError::internal("repeated continuation token"),
        )
    }
    async fn list_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AccessListRequest,
    ) -> Result<crate::proto::r#wafl::r#AccessListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AccessListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AccessListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::AccessService> AccessCalls for N {}
#[allow(async_fn_in_trait)]
pub trait AuditCalls: crate::services::r#wafl::AuditService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AuditGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Audit, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AuditGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Audit::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#AuditListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Audit, ::connectrpc::ConnectError>,
    > + 'a {
        ::paydirt::page::all(
            request,
            |q| &mut q.after,
            move |q| async move {
                let page = self.list_owned(ctx, q).await?;
                Ok((page.items, page.next))
            },
            || ::connectrpc::ConnectError::internal("repeated continuation token"),
        )
    }
    async fn list_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AuditListRequest,
    ) -> Result<crate::proto::r#wafl::r#AuditListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AuditListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AuditListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::AuditService> AuditCalls for N {}
#[allow(async_fn_in_trait)]
pub trait HolderCalls: crate::services::r#wafl::HolderService {
    async fn add_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderAddRequest,
    ) -> Result<crate::proto::r#wafl::r#Holder, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderAddRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .add(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Holder::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Holder, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Holder::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn patch_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderPatchRequest,
    ) -> Result<crate::proto::r#wafl::r#Holder, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderPatchRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .patch(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Holder::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn apply_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderApplyRequest,
    ) -> Result<crate::proto::r#wafl::r#Holder, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderApplyRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .apply(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Holder::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn erase_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderRef,
    ) -> Result<
        crate::proto::r#wafl::r#HolderEraseResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderRef::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .erase(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#HolderEraseResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#HolderListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Holder, ::connectrpc::ConnectError>,
    > + 'a {
        ::paydirt::page::all(
            request,
            |q| &mut q.after,
            move |q| async move {
                let page = self.list_owned(ctx, q).await?;
                Ok((page.items, page.next))
            },
            || ::connectrpc::ConnectError::internal("repeated continuation token"),
        )
    }
    async fn list_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#HolderListRequest,
    ) -> Result<crate::proto::r#wafl::r#HolderListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#HolderListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#HolderListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::HolderService> HolderCalls for N {}
#[allow(async_fn_in_trait)]
pub trait InstanceCalls: crate::services::r#wafl::InstanceService {
    async fn add_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceAddRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceAddRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .add(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn erase_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceRef,
    ) -> Result<
        crate::proto::r#wafl::r#InstanceEraseResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceRef::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .erase(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#InstanceEraseResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#InstanceListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Instance, ::connectrpc::ConnectError>,
    > + 'a {
        ::paydirt::page::all(
            request,
            |q| &mut q.after,
            move |q| async move {
                let page = self.list_owned(ctx, q).await?;
                Ok((page.items, page.next))
            },
            || ::connectrpc::ConnectError::internal("repeated continuation token"),
        )
    }
    async fn list_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceListRequest,
    ) -> Result<
        crate::proto::r#wafl::r#InstanceListResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#InstanceListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn start_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceStartRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceStartRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .start(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn stop_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceStopRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceStopRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .stop(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::InstanceService> InstanceCalls for N {}
#[allow(async_fn_in_trait)]
pub trait TenantCalls: crate::services::r#wafl::TenantService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TenantGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Tenant, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TenantGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Tenant::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn patch_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TenantPatchRequest,
    ) -> Result<crate::proto::r#wafl::r#Tenant, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TenantPatchRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .patch(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Tenant::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn apply_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TenantApplyRequest,
    ) -> Result<crate::proto::r#wafl::r#Tenant, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TenantApplyRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .apply(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Tenant::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#TenantListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Tenant, ::connectrpc::ConnectError>,
    > + 'a {
        ::paydirt::page::all(
            request,
            |q| &mut q.after,
            move |q| async move {
                let page = self.list_owned(ctx, q).await?;
                Ok((page.items, page.next))
            },
            || ::connectrpc::ConnectError::internal("repeated continuation token"),
        )
    }
    async fn list_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TenantListRequest,
    ) -> Result<crate::proto::r#wafl::r#TenantListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TenantListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#TenantListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::TenantService> TenantCalls for N {}
impl<N: server::Server> server::Layer<N> for Sink {
    fn wafl_access(&self, _: &N) -> impl crate::services::r#wafl::AccessService {
        Next(self.0.clone()).wafl_access()
    }
    fn wafl_audit(&self, _: &N) -> impl crate::services::r#wafl::AuditService {
        Next(self.0.clone()).wafl_audit()
    }
    fn wafl_holder(&self, _: &N) -> impl crate::services::r#wafl::HolderService {
        Next(self.0.clone()).wafl_holder()
    }
    fn wafl_instance(&self, _: &N) -> impl crate::services::r#wafl::InstanceService {
        Next(self.0.clone()).wafl_instance()
    }
    fn wafl_tenant(&self, _: &N) -> impl crate::services::r#wafl::TenantService {
        Next(self.0.clone()).wafl_tenant()
    }
}
impl<N: server::Server> server::Layer<N> for Gate {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        crate::services::r#wafl::AccessServiceOverlay {
            next: next.wafl_access(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::HolderService {
        crate::services::r#wafl::HolderServiceOverlay {
            next: next.wafl_holder(),
            layer: self.clone(),
        }
    }
    fn wafl_instance(&self, next: &N) -> impl crate::services::r#wafl::InstanceService {
        crate::services::r#wafl::InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
}
impl<N: server::Server> server::Layer<N> for Audit {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        crate::services::r#wafl::AccessServiceOverlay {
            next: next.wafl_access(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::HolderService {
        crate::services::r#wafl::HolderServiceOverlay {
            next: next.wafl_holder(),
            layer: self.clone(),
        }
    }
    fn wafl_instance(&self, next: &N) -> impl crate::services::r#wafl::InstanceService {
        crate::services::r#wafl::InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
}
impl<N: server::Server> server::Layer<N> for intercept::Layer {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        crate::services::r#wafl::AccessServiceOverlay {
            next: next.wafl_access(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::HolderService {
        crate::services::r#wafl::HolderServiceOverlay {
            next: next.wafl_holder(),
            layer: self.clone(),
        }
    }
    fn wafl_instance(&self, next: &N) -> impl crate::services::r#wafl::InstanceService {
        crate::services::r#wafl::InstanceServiceOverlay {
            next: next.wafl_instance(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
}
pub trait Layers {
    fn interceptors(&self) -> crate::storage::intercept::Layer {
        Default::default()
    }
    fn public_methods(&self) -> &'static [&'static str] {
        &[]
    }
    fn wafl_access(
        &self,
        next: impl crate::services::r#wafl::AccessService,
    ) -> impl crate::services::r#wafl::AccessService {
        next
    }
    fn wafl_audit(
        &self,
        next: impl crate::services::r#wafl::AuditService,
    ) -> impl crate::services::r#wafl::AuditService {
        next
    }
    fn wafl_holder(
        &self,
        next: impl crate::services::r#wafl::HolderService,
    ) -> impl crate::services::r#wafl::HolderService {
        next
    }
    fn wafl_instance(
        &self,
        next: impl crate::services::r#wafl::InstanceService,
    ) -> impl crate::services::r#wafl::InstanceService {
        next
    }
    fn wafl_tenant(
        &self,
        next: impl crate::services::r#wafl::TenantService,
    ) -> impl crate::services::r#wafl::TenantService {
        next
    }
}
struct Application<'a, L>(&'a L);
impl<N: server::Server, L: Layers> server::Layer<N> for Application<'_, L> {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        self.0.wafl_access(next.wafl_access())
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        self.0.wafl_audit(next.wafl_audit())
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::HolderService {
        self.0.wafl_holder(next.wafl_holder())
    }
    fn wafl_instance(&self, next: &N) -> impl crate::services::r#wafl::InstanceService {
        self.0.wafl_instance(next.wafl_instance())
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        self.0.wafl_tenant(next.wafl_tenant())
    }
}
/// The database-backed default stack. Use server::Server::layer to choose
/// the base and layers explicitly, including a stack without a DB Sink.
pub fn mount(router: Router, db: Database, layers: &impl Layers) -> Router {
    use server::Server;
    let stack = server::Unimplemented
        .layer(Sink(db.clone()))
        .layer(Application(layers))
        .layer(layers.interceptors().public())
        .layer(Audit(db.clone()))
        .layer(Gate::new(db, layers.public_methods()));
    server::mount_if(
        router,
        &stack,
        |service| match service {
            "wafl.AccessService" => crate::transport::access::OPTIONS.reads,
            "wafl.AuditService" => crate::transport::access::OPTIONS.reads,
            "wafl.HolderService" => true,
            "wafl.InstanceService" => true,
            "wafl.TenantService" => true,
            _ => false,
        },
    )
}
