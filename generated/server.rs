/// Services supplied by an application, independent of any database.
/// An omitted service answers Unimplemented.
pub trait Server {
    fn wafl_access(&self) -> impl crate::services::r#wafl::r#AccessService {
        crate::services::r#wafl::r#AccessServiceUnimplemented
    }
    fn wafl_app(&self) -> impl crate::services::r#wafl::r#AppService {
        crate::services::r#wafl::r#AppServiceUnimplemented
    }
    fn wafl_app_source(&self) -> impl crate::services::r#wafl::r#AppSourceService {
        crate::services::r#wafl::r#AppSourceServiceUnimplemented
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        crate::services::r#wafl::r#AuditServiceUnimplemented
    }
    fn wafl_controller(&self) -> impl crate::services::r#wafl::r#ControllerService {
        crate::services::r#wafl::r#ControllerServiceUnimplemented
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        crate::services::r#wafl::r#HolderServiceUnimplemented
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        crate::services::r#wafl::r#InstanceServiceUnimplemented
    }
    fn wafl_placement(&self) -> impl crate::services::r#wafl::r#PlacementService {
        crate::services::r#wafl::r#PlacementServiceUnimplemented
    }
    fn wafl_reservation(&self) -> impl crate::services::r#wafl::r#ReservationService {
        crate::services::r#wafl::r#ReservationServiceUnimplemented
    }
    fn wafl_runner(&self) -> impl crate::services::r#wafl::r#RunnerService {
        crate::services::r#wafl::r#RunnerServiceUnimplemented
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        crate::services::r#wafl::r#TenantServiceUnimplemented
    }
    fn wafl_traffic(&self) -> impl crate::services::r#wafl::r#TrafficService {
        crate::services::r#wafl::r#TrafficServiceUnimplemented
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
    fn wafl_app(&self, next: &N) -> impl crate::services::r#wafl::r#AppService {
        next.wafl_app()
    }
    fn wafl_app_source(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::r#AppSourceService {
        next.wafl_app_source()
    }
    fn wafl_audit(&self, next: &N) -> impl crate::services::r#wafl::r#AuditService {
        next.wafl_audit()
    }
    fn wafl_controller(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::r#ControllerService {
        next.wafl_controller()
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
    fn wafl_placement(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::r#PlacementService {
        next.wafl_placement()
    }
    fn wafl_reservation(
        &self,
        next: &N,
    ) -> impl crate::services::r#wafl::r#ReservationService {
        next.wafl_reservation()
    }
    fn wafl_runner(&self, next: &N) -> impl crate::services::r#wafl::r#RunnerService {
        next.wafl_runner()
    }
    fn wafl_tenant(&self, next: &N) -> impl crate::services::r#wafl::r#TenantService {
        next.wafl_tenant()
    }
    fn wafl_traffic(&self, next: &N) -> impl crate::services::r#wafl::r#TrafficService {
        next.wafl_traffic()
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
    fn wafl_app(&self) -> impl crate::services::r#wafl::r#AppService {
        self.layer.wafl_app(&self.next)
    }
    fn wafl_app_source(&self) -> impl crate::services::r#wafl::r#AppSourceService {
        self.layer.wafl_app_source(&self.next)
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        self.layer.wafl_audit(&self.next)
    }
    fn wafl_controller(&self) -> impl crate::services::r#wafl::r#ControllerService {
        self.layer.wafl_controller(&self.next)
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        self.layer.wafl_holder(&self.next)
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        self.layer.wafl_instance(&self.next)
    }
    fn wafl_placement(&self) -> impl crate::services::r#wafl::r#PlacementService {
        self.layer.wafl_placement(&self.next)
    }
    fn wafl_reservation(&self) -> impl crate::services::r#wafl::r#ReservationService {
        self.layer.wafl_reservation(&self.next)
    }
    fn wafl_runner(&self) -> impl crate::services::r#wafl::r#RunnerService {
        self.layer.wafl_runner(&self.next)
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        self.layer.wafl_tenant(&self.next)
    }
    fn wafl_traffic(&self) -> impl crate::services::r#wafl::r#TrafficService {
        self.layer.wafl_traffic(&self.next)
    }
}
impl<T: Server + ?Sized> Server for ::std::sync::Arc<T> {
    fn wafl_access(&self) -> impl crate::services::r#wafl::r#AccessService {
        self.as_ref().wafl_access()
    }
    fn wafl_app(&self) -> impl crate::services::r#wafl::r#AppService {
        self.as_ref().wafl_app()
    }
    fn wafl_app_source(&self) -> impl crate::services::r#wafl::r#AppSourceService {
        self.as_ref().wafl_app_source()
    }
    fn wafl_audit(&self) -> impl crate::services::r#wafl::r#AuditService {
        self.as_ref().wafl_audit()
    }
    fn wafl_controller(&self) -> impl crate::services::r#wafl::r#ControllerService {
        self.as_ref().wafl_controller()
    }
    fn wafl_holder(&self) -> impl crate::services::r#wafl::r#HolderService {
        self.as_ref().wafl_holder()
    }
    fn wafl_instance(&self) -> impl crate::services::r#wafl::r#InstanceService {
        self.as_ref().wafl_instance()
    }
    fn wafl_placement(&self) -> impl crate::services::r#wafl::r#PlacementService {
        self.as_ref().wafl_placement()
    }
    fn wafl_reservation(&self) -> impl crate::services::r#wafl::r#ReservationService {
        self.as_ref().wafl_reservation()
    }
    fn wafl_runner(&self) -> impl crate::services::r#wafl::r#RunnerService {
        self.as_ref().wafl_runner()
    }
    fn wafl_tenant(&self) -> impl crate::services::r#wafl::r#TenantService {
        self.as_ref().wafl_tenant()
    }
    fn wafl_traffic(&self) -> impl crate::services::r#wafl::r#TrafficService {
        self.as_ref().wafl_traffic()
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
    if include("wafl.AppService") {
        router = crate::services::r#wafl::r#AppServiceExt::register(
            ::std::sync::Arc::new(server.wafl_app()),
            router,
        );
    }
    if include("wafl.AppSourceService") {
        router = crate::services::r#wafl::r#AppSourceServiceExt::register(
            ::std::sync::Arc::new(server.wafl_app_source()),
            router,
        );
    }
    if include("wafl.AuditService") {
        router = crate::services::r#wafl::r#AuditServiceExt::register(
            ::std::sync::Arc::new(server.wafl_audit()),
            router,
        );
    }
    if include("wafl.ControllerService") {
        router = crate::services::r#wafl::r#ControllerServiceExt::register(
            ::std::sync::Arc::new(server.wafl_controller()),
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
    if include("wafl.PlacementService") {
        router = crate::services::r#wafl::r#PlacementServiceExt::register(
            ::std::sync::Arc::new(server.wafl_placement()),
            router,
        );
    }
    if include("wafl.ReservationService") {
        router = crate::services::r#wafl::r#ReservationServiceExt::register(
            ::std::sync::Arc::new(server.wafl_reservation()),
            router,
        );
    }
    if include("wafl.RunnerService") {
        router = crate::services::r#wafl::r#RunnerServiceExt::register(
            ::std::sync::Arc::new(server.wafl_runner()),
            router,
        );
    }
    if include("wafl.TenantService") {
        router = crate::services::r#wafl::r#TenantServiceExt::register(
            ::std::sync::Arc::new(server.wafl_tenant()),
            router,
        );
    }
    if include("wafl.TrafficService") {
        router = crate::services::r#wafl::r#TrafficServiceExt::register(
            ::std::sync::Arc::new(server.wafl_traffic()),
            router,
        );
    }
    router
}
