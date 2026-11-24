/// Services supplied by an application, independent of any database.
/// An omitted service answers Unimplemented.
pub trait Server {
    fn wafl_access(&self) -> impl crate::services::r#wafl::r#AccessService {
        crate::services::r#wafl::r#AccessServiceUnimplemented
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        crate::services::r#wafl::r#AuditServiceUnimplemented
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        crate::services::r#wafl::r#HolderServiceUnimplemented
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        crate::services::r#wafl::r#InstanceServiceUnimplemented
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        crate::services::r#wafl::r#TenantServiceUnimplemented
    }
    /// Add an outer layer. The last layer added handles a request first.
    fn layer<L: Layer<Self>>(self, layer: L) -> Overlay<Self, L>
    where
        Self: Sized,
    {
        Overlay { next: self, layer }
    }
}
#[derive(Clone, Copy, Default)]
pub struct Unimplemented;
impl Server for Unimplemented {}
/// Each accessor may replace a service or wrap the service behind it.
pub trait Layer<N: Server + ?Sized> {
    fn wafl_access(&self, next: &N) -> impl crate::services::r#wafl::r#AccessService {
        next.wafl_access()
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::r#AuditService {
        next.wafl_audit()
    }
    fn wafl_holder(&self, next: &N) -> impl crate::services::r#wafl::r#HolderService {
        next.wafl_holder()
    }
    fn wafl_instance(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::r#InstanceService {
        next.wafl_instance()
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::r#TenantService {
        next.wafl_tenant()
    }
}
#[derive(Clone)]
pub struct Overlay<N, L> {
    pub next: N,
    pub layer: L,
}
impl<N: Server, L: Layer<N>> Server for Overlay<N, L> {
    fn wafl_access(&self) -> impl crate::services::r#wafl::r#AccessService {
        self.layer.wafl_access(&self.next)
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        self.layer.wafl_audit(&self.next)
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        self.layer.wafl_holder(&self.next)
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        self.layer.wafl_instance(&self.next)
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        self.layer.wafl_tenant(&self.next)
    }
}
impl<T: Server + ?Sized> Server for ::std::sync::Arc<T> {
    fn wafl_access(&self) -> impl crate::services::r#wafl::r#AccessService {
        self.as_ref().wafl_access()
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        self.as_ref().wafl_audit()
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        self.as_ref().wafl_holder()
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        self.as_ref().wafl_instance()
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        self.as_ref().wafl_tenant()
    }
}
pub fn mount(
    router: ::connectrpc::Router,
    server: &impl Server,
) -> ::connectrpc::Router {
    mount_if(router, server, |_| true)
}
/// Select services at registration; this does not authorize requests.
pub fn mount_if(
    mut router: ::connectrpc::Router,
    server: &impl Server,
    include: impl Fn(&str) -> bool,
) -> ::connectrpc::Router {
    if include("wafl.AccessService") {
        router = crate::services::r#wafl::r#AccessServiceExt::register(
            ::std::sync::Arc::new(server.wafl_access()),
            router,
        );
    }
    if include("wafl.AuditService") {
        router = crate::services::r#wafl::r#AuditServiceExt::register(
            ::std::sync::Arc::new(server.wafl_audit()),
            router,
        );
    }
    if include("wafl.HolderService") {
        router = crate::services::r#wafl::r#HolderServiceExt::register(
            ::std::sync::Arc::new(server.wafl_holder()),
            router,
        );
    }
    if include("wafl.InstanceService") {
        router = crate::services::r#wafl::r#InstanceServiceExt::register(
            ::std::sync::Arc::new(server.wafl_instance()),
            router,
        );
    }
    if include("wafl.TenantService") {
        router = crate::services::r#wafl::r#TenantServiceExt::register(
            ::std::sync::Arc::new(server.wafl_tenant()),
            router,
        );
    }
    router
}
