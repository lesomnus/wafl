pub trait Layers {
    fn wafl_instance(&self) -> impl crate::services::r#wafl::InstanceService {
        crate::services::r#wafl::InstanceServiceUnimplemented
    }
}
pub fn mount(
    router: ::connectrpc::Router,
    layers: &impl Layers,
) -> ::connectrpc::Router {
    let router = crate::services::r#wafl::InstanceServiceExt::register(
        ::std::sync::Arc::new(layers.wafl_instance()),
        router,
    );
    router
}
