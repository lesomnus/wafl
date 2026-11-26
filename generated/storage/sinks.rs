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
    N: crate::services::r#wafl::AppSourceService,
> crate::services::r#wafl::AppSourceServiceLayer<N>
for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::AppSourceGetRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppSource,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppSourceService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AppSource, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::AppSourceListRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppSourceListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppSourceService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AppSourceListResponse, _>(reply)
        }
    }
}
impl<N: crate::services::r#wafl::AppService> crate::services::r#wafl::AppServiceLayer<N>
for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#App, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#App, _>(reply)
        }
    }
    fn patch<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/Patch", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.patch(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#App, _>(reply)
        }
    }
    fn apply<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/Apply", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.apply(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#App, _>(reply)
        }
    }
    fn erase<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppEraseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/Erase", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.erase(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AppEraseResponse, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.AppService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#AppListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::ControllerService,
> crate::services::r#wafl::ControllerServiceLayer<N>
for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::ControllerGetRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Controller,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.ControllerService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Controller, _>(reply)
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
    fn halt<'a>(
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
            let mut call = self.enter("/wafl.InstanceService/Halt", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.halt(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn drain<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceDrainRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Drain", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.drain(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn release<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceReleaseRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Release", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.release(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Instance, _>(reply)
        }
    }
    fn seal<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceSealRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceSealResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Seal", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.seal(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#InstanceSealResponse, _>(reply)
        }
    }
    fn tail(
        &self,
        next: &N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceTailRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<crate::proto::wafl::Log> + Send + use<N>,
            >,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.InstanceService/Tail", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.tail(call.ctx.clone(), request).await;
            call.stream::<crate::proto::r#wafl::r#Log, _>(reply)
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
impl<
    N: crate::services::r#wafl::PlacementService,
> crate::services::r#wafl::PlacementServiceLayer<N>
for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::PlacementGetRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Placement,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.PlacementService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Placement, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::PlacementListRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::PlacementListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.PlacementService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#PlacementListResponse, _>(reply)
        }
    }
    fn release<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::PlacementReleaseRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Placement,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.PlacementService/Release", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.release(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Placement, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::ReservationService,
> crate::services::r#wafl::ReservationServiceLayer<N>
for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::ReservationGetRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Reservation,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.ReservationService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Reservation, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::ReservationListRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::ReservationListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.ReservationService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#ReservationListResponse, _>(reply)
        }
    }
    fn release<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::ReservationReleaseRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::ReservationReleaseResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.ReservationService/Release", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.release(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#ReservationReleaseResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::RunnerService,
> crate::services::r#wafl::RunnerServiceLayer<N> for crate::storage::intercept::Layer {
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::RunnerGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Runner> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.RunnerService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Runner, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::RunnerListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::RunnerListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.RunnerService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#RunnerListResponse, _>(reply)
        }
    }
}
impl<
    N: crate::services::r#wafl::TrafficService,
> crate::services::r#wafl::TrafficServiceLayer<N> for crate::storage::intercept::Layer {
    fn add<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TrafficAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Traffic> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TrafficService/Add", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.add(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Traffic, _>(reply)
        }
    }
    fn get<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TrafficGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Traffic> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TrafficService/Get", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.get(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Traffic, _>(reply)
        }
    }
    fn list<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TrafficListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::TrafficListResponse,
            > + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TrafficService/List", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.list(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#TrafficListResponse, _>(reply)
        }
    }
    fn report<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::TrafficReportRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Traffic> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TrafficService/Report", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.report(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Traffic, _>(reply)
        }
    }
    fn close<'a>(
        &'a self,
        next: &'a N,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::TrafficCloseRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::Traffic> + Send + use<'a, N>,
        >,
    > + Send {
        async move {
            let mut call = self.enter("/wafl.TrafficService/Close", ctx)?;
            call.receive(request.bytes())?;
            let reply = next.close(call.ctx.clone(), request).await;
            call.unary::<crate::proto::r#wafl::r#Traffic, _>(reply)
        }
    }
}
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
mod sink_wafl_app {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::AppService,
    > crate::services::r#wafl::AppServiceLayer<N> for Sink {
        async fn add(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppAddRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#App> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "Add", &request.to_owned_message()).await?,
            )
        }
        async fn apply(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppApplyRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#App> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "Apply", &request.to_owned_message()).await?,
            )
        }
        async fn erase(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppRef>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AppEraseResponse> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "Erase", &request.to_owned_message()).await?,
            )
        }
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#App> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "Get", &request.to_owned_message()).await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AppListResponse> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "List", &request.to_owned_message()).await?,
            )
        }
        async fn patch(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppPatchRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#App> {
            Response::ok(
                self.0.rpc(&ctx, "wafl.App", "Patch", &request.to_owned_message()).await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::AppService,
    > crate::services::r#wafl::AppServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppAddRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
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
                    "wafl.AppAddRequest",
                    "wafl.App",
                    "/wafl.AppService/Add",
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
                        "wafl.App",
                        "Add",
                        self.public.contains(&"/wafl.AppService/Add"),
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
                    .output::<crate::proto::r#wafl::r#App, _>(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppGetRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
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
                    "wafl.AppGetRequest",
                    "wafl.App",
                    "/wafl.AppService/Get",
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
                        "wafl.App",
                        "Get",
                        self.public.contains(&"/wafl.AppService/Get"),
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
                    .output::<crate::proto::r#wafl::r#App, _>(&self.db, &response.body)
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
                crate::proto::wafl::AppPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
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
                    "wafl.AppPatchRequest",
                    "wafl.App",
                    "/wafl.AppService/Patch",
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
                        "wafl.App",
                        "Patch",
                        self.public.contains(&"/wafl.AppService/Patch"),
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
                    .output::<crate::proto::r#wafl::r#App, _>(&self.db, &response.body)
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
                crate::proto::wafl::AppApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
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
                    "wafl.AppApplyRequest",
                    "wafl.App",
                    "/wafl.AppService/Apply",
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
                        "wafl.App",
                        "Apply",
                        self.public.contains(&"/wafl.AppService/Apply"),
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
                    .output::<crate::proto::r#wafl::r#App, _>(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn erase<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppEraseResponse,
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
                    "wafl.AppRef",
                    "wafl.AppEraseResponse",
                    "/wafl.AppService/Erase",
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
                        "wafl.App",
                        "Erase",
                        self.public.contains(&"/wafl.AppService/Erase"),
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
                        crate::proto::r#wafl::r#AppEraseResponse,
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
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppListRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppListResponse,
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
                    "wafl.AppListRequest",
                    "wafl.AppListResponse",
                    "/wafl.AppService/List",
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
                        "wafl.App",
                        "List",
                        self.public.contains(&"/wafl.AppService/List"),
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
                        crate::proto::r#wafl::r#AppListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::AppService,
    > crate::services::r#wafl::AppServiceLayer<N> for Audit {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppAddRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppGetRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
                crate::proto::wafl::AppPatchRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
                crate::proto::wafl::AppApplyRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppRef>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppEraseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
            request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppListRequest>,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.App",
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
mod sink_wafl_app_source {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::AppSourceService,
    > crate::services::r#wafl::AppSourceServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppSourceGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AppSource> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.AppSource", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#AppSourceListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#AppSourceListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.AppSource", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::AppSourceService,
    > crate::services::r#wafl::AppSourceServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AppSourceGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppSource,
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
                    "wafl.AppSourceGetRequest",
                    "wafl.AppSource",
                    "/wafl.AppSourceService/Get",
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
                        "wafl.AppSource",
                        "Get",
                        self.public.contains(&"/wafl.AppSourceService/Get"),
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
                        crate::proto::r#wafl::r#AppSource,
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
                crate::proto::wafl::AppSourceListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppSourceListResponse,
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
                    "wafl.AppSourceListRequest",
                    "wafl.AppSourceListResponse",
                    "/wafl.AppSourceService/List",
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
                        "wafl.AppSource",
                        "List",
                        self.public.contains(&"/wafl.AppSourceService/List"),
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
                        crate::proto::r#wafl::r#AppSourceListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::AppSourceService,
    > crate::services::r#wafl::AppSourceServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::AppSourceGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppSource,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.AppSource",
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
                crate::proto::wafl::AppSourceListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::AppSourceListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.AppSource",
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
mod sink_wafl_controller {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::ControllerService,
    > crate::services::r#wafl::ControllerServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#ControllerGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Controller> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Controller", "Get", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::ControllerService,
    > crate::services::r#wafl::ControllerServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ControllerGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Controller,
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
                    "wafl.ControllerGetRequest",
                    "wafl.Controller",
                    "/wafl.ControllerService/Get",
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
                        "wafl.Controller",
                        "Get",
                        self.public.contains(&"/wafl.ControllerService/Get"),
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
                        crate::proto::r#wafl::r#Controller,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::ControllerService,
    > crate::services::r#wafl::ControllerServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ControllerGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Controller,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Controller",
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
        fn halt<'a>(
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
                    "/wafl.InstanceService/Halt",
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
                        "Halt",
                        self.public.contains(&"/wafl.InstanceService/Halt"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.halt(ctx.clone(), request).await {
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
        fn drain<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceDrainRequest,
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
                    "wafl.InstanceDrainRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Drain",
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
                        "Drain",
                        self.public.contains(&"/wafl.InstanceService/Drain"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.drain(ctx.clone(), request).await {
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
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceReleaseRequest,
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
                    "wafl.InstanceReleaseRequest",
                    "wafl.Instance",
                    "/wafl.InstanceService/Release",
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
                        "Release",
                        self.public.contains(&"/wafl.InstanceService/Release"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.release(ctx.clone(), request).await {
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
        fn seal<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceSealRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceSealResponse,
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
                    "wafl.InstanceSealRequest",
                    "wafl.InstanceSealResponse",
                    "/wafl.InstanceService/Seal",
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
                        "Seal",
                        self.public.contains(&"/wafl.InstanceService/Seal"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.seal(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#InstanceSealResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn tail(
            &self,
            next: &N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceTailRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                ::connectrpc::ServiceStream<
                    impl ::connectrpc::Encodable<crate::proto::wafl::Log> + Send + use<N>,
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
                    "wafl.InstanceTailRequest",
                    "wafl.Log",
                    "/wafl.InstanceService/Tail",
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
                        "Tail",
                        self.public.contains(&"/wafl.InstanceService/Tail"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.tail(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                Ok(
                    response
                        .map_body(|body| {
                            attempt
                                .stream::<
                                    crate::proto::r#wafl::r#Log,
                                    _,
                                >(self.db.clone(), Database::enforce_presence(&ctx, body))
                        }),
                )
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
        fn halt<'a>(
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
                        "Halt",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.halt(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn drain<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceDrainRequest,
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
                        "Drain",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.drain(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceReleaseRequest,
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
                        "Release",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.release(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn seal<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::InstanceSealRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::InstanceSealResponse,
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
                        "Seal",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.seal(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_placement {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::PlacementService,
    > crate::services::r#wafl::PlacementServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#PlacementGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Placement> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Placement", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#PlacementListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#PlacementListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Placement", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::PlacementService,
    > crate::services::r#wafl::PlacementServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::PlacementGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Placement,
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
                    "wafl.PlacementGetRequest",
                    "wafl.Placement",
                    "/wafl.PlacementService/Get",
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
                        "wafl.Placement",
                        "Get",
                        self.public.contains(&"/wafl.PlacementService/Get"),
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
                        crate::proto::r#wafl::r#Placement,
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
                crate::proto::wafl::PlacementListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::PlacementListResponse,
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
                    "wafl.PlacementListRequest",
                    "wafl.PlacementListResponse",
                    "/wafl.PlacementService/List",
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
                        "wafl.Placement",
                        "List",
                        self.public.contains(&"/wafl.PlacementService/List"),
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
                        crate::proto::r#wafl::r#PlacementListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::PlacementReleaseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Placement,
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
                    "wafl.PlacementReleaseRequest",
                    "wafl.Placement",
                    "/wafl.PlacementService/Release",
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
                        "wafl.Placement",
                        "Release",
                        self.public.contains(&"/wafl.PlacementService/Release"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.release(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Placement,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::PlacementService,
    > crate::services::r#wafl::PlacementServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::PlacementGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Placement,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Placement",
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
                crate::proto::wafl::PlacementListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::PlacementListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Placement",
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
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::PlacementReleaseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Placement,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Placement",
                        "Release",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.release(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_reservation {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::ReservationService,
    > crate::services::r#wafl::ReservationServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#ReservationGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Reservation> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Reservation", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#ReservationListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#ReservationListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Reservation", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::ReservationService,
    > crate::services::r#wafl::ReservationServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ReservationGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Reservation,
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
                    "wafl.ReservationGetRequest",
                    "wafl.Reservation",
                    "/wafl.ReservationService/Get",
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
                        "wafl.Reservation",
                        "Get",
                        self.public.contains(&"/wafl.ReservationService/Get"),
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
                        crate::proto::r#wafl::r#Reservation,
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
                crate::proto::wafl::ReservationListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::ReservationListResponse,
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
                    "wafl.ReservationListRequest",
                    "wafl.ReservationListResponse",
                    "/wafl.ReservationService/List",
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
                        "wafl.Reservation",
                        "List",
                        self.public.contains(&"/wafl.ReservationService/List"),
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
                        crate::proto::r#wafl::r#ReservationListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ReservationReleaseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::ReservationReleaseResponse,
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
                    "wafl.ReservationReleaseRequest",
                    "wafl.ReservationReleaseResponse",
                    "/wafl.ReservationService/Release",
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
                        "wafl.Reservation",
                        "Release",
                        self.public.contains(&"/wafl.ReservationService/Release"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.release(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#ReservationReleaseResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::ReservationService,
    > crate::services::r#wafl::ReservationServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ReservationGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Reservation,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Reservation",
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
                crate::proto::wafl::ReservationListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::ReservationListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Reservation",
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
        fn release<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::ReservationReleaseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::ReservationReleaseResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Reservation",
                        "Release",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.release(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
    }
}
mod sink_wafl_runner {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::RunnerService,
    > crate::services::r#wafl::RunnerServiceLayer<N> for Sink {
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#RunnerGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Runner> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Runner", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#RunnerListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#RunnerListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Runner", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::RunnerService,
    > crate::services::r#wafl::RunnerServiceLayer<N> for Gate {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::RunnerGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Runner,
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
                    "wafl.RunnerGetRequest",
                    "wafl.Runner",
                    "/wafl.RunnerService/Get",
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
                        "wafl.Runner",
                        "Get",
                        self.public.contains(&"/wafl.RunnerService/Get"),
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
                        crate::proto::r#wafl::r#Runner,
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
                crate::proto::wafl::RunnerListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::RunnerListResponse,
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
                    "wafl.RunnerListRequest",
                    "wafl.RunnerListResponse",
                    "/wafl.RunnerService/List",
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
                        "wafl.Runner",
                        "List",
                        self.public.contains(&"/wafl.RunnerService/List"),
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
                        crate::proto::r#wafl::r#RunnerListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::RunnerService,
    > crate::services::r#wafl::RunnerServiceLayer<N> for Audit {
        fn get<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::RunnerGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Runner,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Runner",
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
                crate::proto::wafl::RunnerListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::RunnerListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Runner",
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
mod sink_wafl_traffic {
    use super::*;
    pub struct Sink(pub Database);
    #[allow(refining_impl_trait)]
    impl<
        N: crate::services::r#wafl::TrafficService,
    > crate::services::r#wafl::TrafficServiceLayer<N> for Sink {
        async fn add(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TrafficAddRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Traffic> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Traffic", "Add", &request.to_owned_message())
                    .await?,
            )
        }
        async fn get(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TrafficGetRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#Traffic> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Traffic", "Get", &request.to_owned_message())
                    .await?,
            )
        }
        async fn list(
            &self,
            _: &N,
            ctx: RequestContext,
            request: ServiceRequest<'_, crate::proto::r#wafl::r#TrafficListRequest>,
        ) -> ServiceResult<crate::proto::r#wafl::r#TrafficListResponse> {
            Response::ok(
                self
                    .0
                    .rpc(&ctx, "wafl.Traffic", "List", &request.to_owned_message())
                    .await?,
            )
        }
    }
    impl<
        N: crate::services::r#wafl::TrafficService,
    > crate::services::r#wafl::TrafficServiceLayer<N> for Gate {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
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
                    "wafl.TrafficAddRequest",
                    "wafl.Traffic",
                    "/wafl.TrafficService/Add",
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
                        "wafl.Traffic",
                        "Add",
                        self.public.contains(&"/wafl.TrafficService/Add"),
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
                        crate::proto::r#wafl::r#Traffic,
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
                crate::proto::wafl::TrafficGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
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
                    "wafl.TrafficGetRequest",
                    "wafl.Traffic",
                    "/wafl.TrafficService/Get",
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
                        "wafl.Traffic",
                        "Get",
                        self.public.contains(&"/wafl.TrafficService/Get"),
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
                        crate::proto::r#wafl::r#Traffic,
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
                crate::proto::wafl::TrafficListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::TrafficListResponse,
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
                    "wafl.TrafficListRequest",
                    "wafl.TrafficListResponse",
                    "/wafl.TrafficService/List",
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
                        "wafl.Traffic",
                        "List",
                        self.public.contains(&"/wafl.TrafficService/List"),
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
                        crate::proto::r#wafl::r#TrafficListResponse,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn report<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficReportRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
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
                    "wafl.TrafficReportRequest",
                    "wafl.Traffic",
                    "/wafl.TrafficService/Report",
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
                        "wafl.Traffic",
                        "Report",
                        self.public.contains(&"/wafl.TrafficService/Report"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.report(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Traffic,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
        fn close<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficCloseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
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
                    "wafl.TrafficCloseRequest",
                    "wafl.Traffic",
                    "/wafl.TrafficService/Close",
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
                        "wafl.Traffic",
                        "Close",
                        self.public.contains(&"/wafl.TrafficService/Close"),
                        false,
                        false,
                    )
                    .await
                {
                    Ok(c) => c,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                let response = match next.close(ctx.clone(), request).await {
                    Ok(v) => v,
                    Err(e) => return Err(attempt.failed(&self.db, e).await),
                };
                attempt
                    .output::<
                        crate::proto::r#wafl::r#Traffic,
                        _,
                    >(&self.db, &response.body)
                    .await?;
                Ok(response)
            }
        }
    }
    impl<
        N: crate::services::r#wafl::TrafficService,
    > crate::services::r#wafl::TrafficServiceLayer<N> for Audit {
        fn add<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficAddRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Traffic",
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
                crate::proto::wafl::TrafficGetRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Traffic",
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
                crate::proto::wafl::TrafficListRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::TrafficListResponse,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Traffic",
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
        fn report<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficReportRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Traffic",
                        "Report",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.report(ctx, request).await;
                if let Some(guard) = guard {
                    guard.finish(result.is_ok()).await?;
                }
                result
            }
        }
        fn close<'a>(
            &'a self,
            next: &'a N,
            ctx: ::connectrpc::RequestContext,
            request: ::connectrpc::ServiceRequest<
                '_,
                crate::proto::wafl::TrafficCloseRequest,
            >,
        ) -> impl ::std::future::Future<
            Output = ::connectrpc::ServiceResult<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::Traffic,
                > + Send + use<'a, N>,
            >,
        > + Send {
            async move {
                let mut ctx = ctx;
                let guard = self
                    .0
                    .begin_request(
                        &mut ctx,
                        "wafl.Traffic",
                        "Close",
                        || request.to_owned_message(),
                    )
                    .await?;
                let result = next.close(ctx, request).await;
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
    pub fn wafl_app(&self) -> impl crate::services::r#wafl::AppService + use<> {
        crate::services::r#wafl::AppServiceOverlay {
            next: crate::services::r#wafl::AppServiceUnimplemented,
            layer: sink_wafl_app::Sink(self.0.clone()),
        }
    }
    pub fn wafl_app_source(
        &self,
    ) -> impl crate::services::r#wafl::AppSourceService + use<> {
        crate::services::r#wafl::AppSourceServiceOverlay {
            next: crate::services::r#wafl::AppSourceServiceUnimplemented,
            layer: sink_wafl_app_source::Sink(self.0.clone()),
        }
    }
    pub fn wafl_audit(&self) -> impl crate::services::r#wafl::AuditService + use<> {
        crate::services::r#wafl::AuditServiceOverlay {
            next: crate::services::r#wafl::AuditServiceUnimplemented,
            layer: sink_wafl_audit::Sink(self.0.clone()),
        }
    }
    pub fn wafl_controller(
        &self,
    ) -> impl crate::services::r#wafl::ControllerService + use<> {
        crate::services::r#wafl::ControllerServiceOverlay {
            next: crate::services::r#wafl::ControllerServiceUnimplemented,
            layer: sink_wafl_controller::Sink(self.0.clone()),
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
    pub fn wafl_placement(
        &self,
    ) -> impl crate::services::r#wafl::PlacementService + use<> {
        crate::services::r#wafl::PlacementServiceOverlay {
            next: crate::services::r#wafl::PlacementServiceUnimplemented,
            layer: sink_wafl_placement::Sink(self.0.clone()),
        }
    }
    pub fn wafl_reservation(
        &self,
    ) -> impl crate::services::r#wafl::ReservationService + use<> {
        crate::services::r#wafl::ReservationServiceOverlay {
            next: crate::services::r#wafl::ReservationServiceUnimplemented,
            layer: sink_wafl_reservation::Sink(self.0.clone()),
        }
    }
    pub fn wafl_runner(&self) -> impl crate::services::r#wafl::RunnerService + use<> {
        crate::services::r#wafl::RunnerServiceOverlay {
            next: crate::services::r#wafl::RunnerServiceUnimplemented,
            layer: sink_wafl_runner::Sink(self.0.clone()),
        }
    }
    pub fn wafl_tenant(&self) -> impl crate::services::r#wafl::TenantService + use<> {
        crate::services::r#wafl::TenantServiceOverlay {
            next: crate::services::r#wafl::TenantServiceUnimplemented,
            layer: sink_wafl_tenant::Sink(self.0.clone()),
        }
    }
    pub fn wafl_traffic(&self) -> impl crate::services::r#wafl::TrafficService + use<> {
        crate::services::r#wafl::TrafficServiceOverlay {
            next: crate::services::r#wafl::TrafficServiceUnimplemented,
            layer: sink_wafl_traffic::Sink(self.0.clone()),
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
pub trait AppCalls: crate::services::r#wafl::AppService {
    async fn add_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppAddRequest,
    ) -> Result<crate::proto::r#wafl::r#App, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppAddRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .add(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#App::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppGetRequest,
    ) -> Result<crate::proto::r#wafl::r#App, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#App::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn patch_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppPatchRequest,
    ) -> Result<crate::proto::r#wafl::r#App, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppPatchRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .patch(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#App::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn apply_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppApplyRequest,
    ) -> Result<crate::proto::r#wafl::r#App, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppApplyRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .apply(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#App::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn erase_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppRef,
    ) -> Result<crate::proto::r#wafl::r#AppEraseResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppRef::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .erase(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AppEraseResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#AppListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::App, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#AppListRequest,
    ) -> Result<crate::proto::r#wafl::r#AppListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AppListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::AppService> AppCalls for N {}
#[allow(async_fn_in_trait)]
pub trait AppSourceCalls: crate::services::r#wafl::AppSourceService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#AppSourceGetRequest,
    ) -> Result<crate::proto::r#wafl::r#AppSource, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppSourceGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AppSource::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#AppSourceListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::AppSource, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#AppSourceListRequest,
    ) -> Result<
        crate::proto::r#wafl::r#AppSourceListResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#AppSourceListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#AppSourceListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::AppSourceService> AppSourceCalls for N {}
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
pub trait ControllerCalls: crate::services::r#wafl::ControllerService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#ControllerGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Controller, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#ControllerGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Controller::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::ControllerService> ControllerCalls for N {}
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
    async fn halt_owned(
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
            .halt(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn drain_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceDrainRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceDrainRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .drain(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn release_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceReleaseRequest,
    ) -> Result<crate::proto::r#wafl::r#Instance, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceReleaseRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .release(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Instance::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn seal_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#InstanceSealRequest,
    ) -> Result<
        crate::proto::r#wafl::r#InstanceSealResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#InstanceSealRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .seal(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#InstanceSealResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::InstanceService> InstanceCalls for N {}
#[allow(async_fn_in_trait)]
pub trait PlacementCalls: crate::services::r#wafl::PlacementService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#PlacementGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Placement, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#PlacementGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Placement::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#PlacementListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Placement, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#PlacementListRequest,
    ) -> Result<
        crate::proto::r#wafl::r#PlacementListResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#PlacementListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#PlacementListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn release_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#PlacementReleaseRequest,
    ) -> Result<crate::proto::r#wafl::r#Placement, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#PlacementReleaseRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .release(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Placement::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::PlacementService> PlacementCalls for N {}
#[allow(async_fn_in_trait)]
pub trait ReservationCalls: crate::services::r#wafl::ReservationService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#ReservationGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Reservation, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#ReservationGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Reservation::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#ReservationListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Reservation, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#ReservationListRequest,
    ) -> Result<
        crate::proto::r#wafl::r#ReservationListResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#ReservationListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#ReservationListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn release_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#ReservationReleaseRequest,
    ) -> Result<
        crate::proto::r#wafl::r#ReservationReleaseResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#ReservationReleaseRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .release(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#ReservationReleaseResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::ReservationService> ReservationCalls for N {}
#[allow(async_fn_in_trait)]
pub trait RunnerCalls: crate::services::r#wafl::RunnerService {
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#RunnerGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Runner, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#RunnerGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Runner::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#RunnerListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Runner, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#RunnerListRequest,
    ) -> Result<crate::proto::r#wafl::r#RunnerListResponse, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#RunnerListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#RunnerListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::RunnerService> RunnerCalls for N {}
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
#[allow(async_fn_in_trait)]
pub trait TrafficCalls: crate::services::r#wafl::TrafficService {
    async fn add_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TrafficAddRequest,
    ) -> Result<crate::proto::r#wafl::r#Traffic, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TrafficAddRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .add(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Traffic::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn get_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TrafficGetRequest,
    ) -> Result<crate::proto::r#wafl::r#Traffic, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TrafficGetRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .get(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Traffic::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    fn list_all<'a>(
        &'a self,
        ctx: &'a RequestContext,
        request: &crate::proto::r#wafl::r#TrafficListRequest,
    ) -> impl ::paydirt::page::Stream<
        Item = Result<crate::proto::r#wafl::Traffic, ::connectrpc::ConnectError>,
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
        req: crate::proto::r#wafl::r#TrafficListRequest,
    ) -> Result<
        crate::proto::r#wafl::r#TrafficListResponse,
        ::connectrpc::ConnectError,
    > {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TrafficListRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .list(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#TrafficListResponse::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn report_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TrafficReportRequest,
    ) -> Result<crate::proto::r#wafl::r#Traffic, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TrafficReportRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .report(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Traffic::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
    async fn close_owned(
        &self,
        ctx: &RequestContext,
        req: crate::proto::r#wafl::r#TrafficCloseRequest,
    ) -> Result<crate::proto::r#wafl::r#Traffic, ::connectrpc::ConnectError> {
        use buffa::{Message, HasMessageView};
        use connectrpc::Encodable;
        let bytes = req.encode_to_bytes();
        let view = crate::proto::r#wafl::r#TrafficCloseRequest::decode_view(&bytes)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream request",
            ))?;
        let reply = self
            .close(ctx.clone(), ServiceRequest::from_parts(&view, &bytes))
            .await?;
        let wire = reply.body.encode(connectrpc::CodecFormat::Proto)?;
        crate::proto::r#wafl::r#Traffic::decode_from_slice(&wire)
            .map_err(|_| connectrpc::ConnectError::internal(
                "invalid downstream response",
            ))
    }
}
impl<N: crate::services::r#wafl::TrafficService> TrafficCalls for N {}
impl<N: server::Server> server::Layer<N> for Sink {
    fn wafl_access(&self, _: &N) -> impl crate::services::r#wafl::AccessService {
        Next(self.0.clone()).wafl_access()
    }
    fn wafl_app(&self, _: &N) -> impl crate::services::r#wafl::AppService {
        Next(self.0.clone()).wafl_app()
    }
    fn wafl_app_source(&self, _: &N) -> impl crate::services::r#wafl::AppSourceService {
        Next(self.0.clone()).wafl_app_source()
    }
    fn wafl_audit(&self, _: &N) -> impl crate::services::r#wafl::AuditService {
        Next(self.0.clone()).wafl_audit()
    }
    fn wafl_controller(&self, _: &N) -> impl crate::services::r#wafl::ControllerService {
        Next(self.0.clone()).wafl_controller()
    }
    fn wafl_holder(&self, _: &N) -> impl crate::services::r#wafl::HolderService {
        Next(self.0.clone()).wafl_holder()
    }
    fn wafl_instance(&self, _: &N) -> impl crate::services::r#wafl::InstanceService {
        Next(self.0.clone()).wafl_instance()
    }
    fn wafl_placement(&self, _: &N) -> impl crate::services::r#wafl::PlacementService {
        Next(self.0.clone()).wafl_placement()
    }
    fn wafl_reservation(
        &self,
        _: &N,
    ) -> impl crate::services::r#wafl::ReservationService {
        Next(self.0.clone()).wafl_reservation()
    }
    fn wafl_runner(&self, _: &N) -> impl crate::services::r#wafl::RunnerService {
        Next(self.0.clone()).wafl_runner()
    }
    fn wafl_tenant(&self, _: &N) -> impl crate::services::r#wafl::TenantService {
        Next(self.0.clone()).wafl_tenant()
    }
    fn wafl_traffic(&self, _: &N) -> impl crate::services::r#wafl::TrafficService {
        Next(self.0.clone()).wafl_traffic()
    }
}
impl<N: server::Server> server::Layer<N> for Gate {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        crate::services::r#wafl::AccessServiceOverlay {
            next: next.wafl_access(),
            layer: self.clone(),
        }
    }
    fn wafl_app(&self, next: &N) -> impl crate::services::r#wafl::AppService {
        crate::services::r#wafl::AppServiceOverlay {
            next: next.wafl_app(),
            layer: self.clone(),
        }
    }
    fn wafl_app_source(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::AppSourceService {
        crate::services::r#wafl::AppSourceServiceOverlay {
            next: next.wafl_app_source(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_controller(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ControllerService {
        crate::services::r#wafl::ControllerServiceOverlay {
            next: next.wafl_controller(),
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
    fn wafl_placement(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::PlacementService {
        crate::services::r#wafl::PlacementServiceOverlay {
            next: next.wafl_placement(),
            layer: self.clone(),
        }
    }
    fn wafl_reservation(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ReservationService {
        crate::services::r#wafl::ReservationServiceOverlay {
            next: next.wafl_reservation(),
            layer: self.clone(),
        }
    }
    fn wafl_runner(&self, next: &N) -> impl crate::services::r#wafl::RunnerService {
        crate::services::r#wafl::RunnerServiceOverlay {
            next: next.wafl_runner(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
    fn wafl_traffic(&self, next: &N) -> impl crate::services::r#wafl::TrafficService {
        crate::services::r#wafl::TrafficServiceOverlay {
            next: next.wafl_traffic(),
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
    fn wafl_app(&self, next: &N) -> impl crate::services::r#wafl::AppService {
        crate::services::r#wafl::AppServiceOverlay {
            next: next.wafl_app(),
            layer: self.clone(),
        }
    }
    fn wafl_app_source(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::AppSourceService {
        crate::services::r#wafl::AppSourceServiceOverlay {
            next: next.wafl_app_source(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_controller(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ControllerService {
        crate::services::r#wafl::ControllerServiceOverlay {
            next: next.wafl_controller(),
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
    fn wafl_placement(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::PlacementService {
        crate::services::r#wafl::PlacementServiceOverlay {
            next: next.wafl_placement(),
            layer: self.clone(),
        }
    }
    fn wafl_reservation(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ReservationService {
        crate::services::r#wafl::ReservationServiceOverlay {
            next: next.wafl_reservation(),
            layer: self.clone(),
        }
    }
    fn wafl_runner(&self, next: &N) -> impl crate::services::r#wafl::RunnerService {
        crate::services::r#wafl::RunnerServiceOverlay {
            next: next.wafl_runner(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
    fn wafl_traffic(&self, next: &N) -> impl crate::services::r#wafl::TrafficService {
        crate::services::r#wafl::TrafficServiceOverlay {
            next: next.wafl_traffic(),
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
    fn wafl_app(&self, next: &N) -> impl crate::services::r#wafl::AppService {
        crate::services::r#wafl::AppServiceOverlay {
            next: next.wafl_app(),
            layer: self.clone(),
        }
    }
    fn wafl_app_source(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::AppSourceService {
        crate::services::r#wafl::AppSourceServiceOverlay {
            next: next.wafl_app_source(),
            layer: self.clone(),
        }
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        crate::services::r#wafl::AuditServiceOverlay {
            next: next.wafl_audit(),
            layer: self.clone(),
        }
    }
    fn wafl_controller(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ControllerService {
        crate::services::r#wafl::ControllerServiceOverlay {
            next: next.wafl_controller(),
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
    fn wafl_placement(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::PlacementService {
        crate::services::r#wafl::PlacementServiceOverlay {
            next: next.wafl_placement(),
            layer: self.clone(),
        }
    }
    fn wafl_reservation(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ReservationService {
        crate::services::r#wafl::ReservationServiceOverlay {
            next: next.wafl_reservation(),
            layer: self.clone(),
        }
    }
    fn wafl_runner(&self, next: &N) -> impl crate::services::r#wafl::RunnerService {
        crate::services::r#wafl::RunnerServiceOverlay {
            next: next.wafl_runner(),
            layer: self.clone(),
        }
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        crate::services::r#wafl::TenantServiceOverlay {
            next: next.wafl_tenant(),
            layer: self.clone(),
        }
    }
    fn wafl_traffic(&self, next: &N) -> impl crate::services::r#wafl::TrafficService {
        crate::services::r#wafl::TrafficServiceOverlay {
            next: next.wafl_traffic(),
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
    fn wafl_app(
        &self,
        next: impl crate::services::r#wafl::AppService,
    ) -> impl crate::services::r#wafl::AppService {
        next
    }
    fn wafl_app_source(
        &self,
        next: impl crate::services::r#wafl::AppSourceService,
    ) -> impl crate::services::r#wafl::AppSourceService {
        next
    }
    fn wafl_audit(
        &self,
        next: impl crate::services::r#wafl::AuditService,
    ) -> impl crate::services::r#wafl::AuditService {
        next
    }
    fn wafl_controller(
        &self,
        next: impl crate::services::r#wafl::ControllerService,
    ) -> impl crate::services::r#wafl::ControllerService {
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
    fn wafl_placement(
        &self,
        next: impl crate::services::r#wafl::PlacementService,
    ) -> impl crate::services::r#wafl::PlacementService {
        next
    }
    fn wafl_reservation(
        &self,
        next: impl crate::services::r#wafl::ReservationService,
    ) -> impl crate::services::r#wafl::ReservationService {
        next
    }
    fn wafl_runner(
        &self,
        next: impl crate::services::r#wafl::RunnerService,
    ) -> impl crate::services::r#wafl::RunnerService {
        next
    }
    fn wafl_tenant(
        &self,
        next: impl crate::services::r#wafl::TenantService,
    ) -> impl crate::services::r#wafl::TenantService {
        next
    }
    fn wafl_traffic(
        &self,
        next: impl crate::services::r#wafl::TrafficService,
    ) -> impl crate::services::r#wafl::TrafficService {
        next
    }
}
struct Application<'a, L>(&'a L);
impl<N: server::Server, L: Layers> server::Layer<N> for Application<'_, L> {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::AccessService {
        self.0.wafl_access(next.wafl_access())
    }
    fn wafl_app(&self, next: &N) -> impl crate::services::r#wafl::AppService {
        self.0.wafl_app(next.wafl_app())
    }
    fn wafl_app_source(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::AppSourceService {
        self.0.wafl_app_source(next.wafl_app_source())
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::AuditService {
        self.0.wafl_audit(next.wafl_audit())
    }
    fn wafl_controller(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ControllerService {
        self.0.wafl_controller(next.wafl_controller())
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::HolderService {
        self.0.wafl_holder(next.wafl_holder())
    }
    fn wafl_instance(&self, next: &N) -> impl crate::services::r#wafl::InstanceService {
        self.0.wafl_instance(next.wafl_instance())
    }
    fn wafl_placement(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::PlacementService {
        self.0.wafl_placement(next.wafl_placement())
    }
    fn wafl_reservation(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::ReservationService {
        self.0.wafl_reservation(next.wafl_reservation())
    }
    fn wafl_runner(&self, next: &N) -> impl crate::services::r#wafl::RunnerService {
        self.0.wafl_runner(next.wafl_runner())
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::TenantService {
        self.0.wafl_tenant(next.wafl_tenant())
    }
    fn wafl_traffic(&self, next: &N) -> impl crate::services::r#wafl::TrafficService {
        self.0.wafl_traffic(next.wafl_traffic())
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
            "wafl.AppService" => true,
            "wafl.AppSourceService" => true,
            "wafl.AuditService" => crate::transport::access::OPTIONS.reads,
            "wafl.ControllerService" => true,
            "wafl.HolderService" => true,
            "wafl.InstanceService" => true,
            "wafl.PlacementService" => true,
            "wafl.ReservationService" => true,
            "wafl.RunnerService" => true,
            "wafl.TenantService" => true,
            "wafl.TrafficService" => true,
            _ => false,
        },
    )
}
