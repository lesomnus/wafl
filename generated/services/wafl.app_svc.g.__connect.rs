///Shorthand for `OwnedView<AppAddRequestView<'static>>`.
pub type OwnedAppAddRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppAddRequestView<'static>,
>;
///Shorthand for `OwnedView<AppView<'static>>`.
pub type OwnedAppView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppView<'static>,
>;
///Shorthand for `OwnedView<AppGetRequestView<'static>>`.
pub type OwnedAppGetRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppGetRequestView<'static>,
>;
///Shorthand for `OwnedView<AppPatchRequestView<'static>>`.
pub type OwnedAppPatchRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppPatchRequestView<'static>,
>;
///Shorthand for `OwnedView<AppApplyRequestView<'static>>`.
pub type OwnedAppApplyRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppApplyRequestView<'static>,
>;
///Shorthand for `OwnedView<AppRefView<'static>>`.
pub type OwnedAppRefView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppRefView<'static>,
>;
///Shorthand for `OwnedView<AppEraseResponseView<'static>>`.
pub type OwnedAppEraseResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppEraseResponseView<'static>,
>;
///Shorthand for `OwnedView<AppListRequestView<'static>>`.
pub type OwnedAppListRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppListRequestView<'static>,
>;
///Shorthand for `OwnedView<AppListResponseView<'static>>`.
pub type OwnedAppListResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppListResponseView<'static>,
>;
///Shorthand for `OwnedView<AppTailRequestView<'static>>`.
pub type OwnedAppTailRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppTailRequestView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::wafl::App>
for crate::proto::wafl::__buffa::view::AppView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::App>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AppEraseResponse>
for crate::proto::wafl::__buffa::view::AppEraseResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AppEraseResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppEraseResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AppListResponse>
for crate::proto::wafl::__buffa::view::AppListResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AppListResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AppListResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const APP_SERVICE_SERVICE_NAME: &str = "wafl.AppService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Add` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_ADD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Add",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Get` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_GET_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Get",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Patch` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_PATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Patch",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Apply` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_APPLY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Apply",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Erase` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_ERASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Erase",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `List` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_LIST_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/List",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Tail` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const APP_SERVICE_TAIL_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AppService/Tail",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for AppService.
///
/// # Implementing handlers
///
/// Implement methods with plain `async fn`; the returned future satisfies
/// the `Send` bound automatically.
///
/// **Unary and server-streaming requests** arrive as
/// [`ServiceRequest<'_, Req>`](::connectrpc::ServiceRequest): a zero-copy
/// view of the request plus its body, valid for the duration of the call.
/// Fields are read directly (`request.name` is a `&str` into the decoded
/// buffer) and the borrow may be held across `.await` points. Anything
/// that must outlive the call — `tokio::spawn`, channels, server state,
/// or data captured by a returned response stream — takes owned data:
/// call `request.to_owned_message()` (or copy the specific fields)
/// first.
///
/// **Client-streaming and bidi requests** arrive as
/// [`InboundStream<Req>`](::connectrpc::InboundStream) — a
/// `ServiceStream` of [`StreamMessage`](::connectrpc::StreamMessage)s.
/// Each item owns its decoded buffer and is `Send + 'static`, so items
/// can be buffered or moved into spawned tasks; read fields zero-copy
/// through the generated accessor methods (`item.name()`) or `.view()`,
/// convert with `.to_owned_message()`, or yield an item back unchanged —
/// `StreamMessage<M>` implements `Encodable<M>`.
///
/// Request types resolved through `extern_path` (e.g. well-known types
/// from another crate) use the same wrappers; the crate that owns the
/// type must be generated with buffa ≥ 0.8.0 and views enabled so the
/// backing `HasMessageView` impl exists.
///
/// The `impl Encodable<Out>` return bound accepts the owned `Out`, the
/// generated `OutView<'_>` / `OwnedOutView`,
/// [`MaybeBorrowed`](::connectrpc::MaybeBorrowed), or
/// [`PreEncoded`](::connectrpc::PreEncoded) for handlers that encode a
/// non-`'static` view internally and pass the bytes across the handler
/// boundary. View bodies are not emitted for output types mapped via
/// `extern_path` (the impl would be an orphan); return owned for
/// WKT/extern outputs.
///
/// Server-streaming and bidi-streaming methods return
/// `ServiceStream<impl Encodable<Out> + Send + use<Self>>`. The
/// `use<Self>` precise-capturing clause excludes `&self`'s lifetime and
/// the request's lifetime (unary methods use `use<'a, Self>` and may
/// borrow from `&self`), so stream items must be `'static` and cannot
/// borrow from the request. To stream view-encoded data, encode each
/// item inside the stream body and yield
/// [`PreEncoded`](::connectrpc::PreEncoded) — see its `# Streaming
/// example` doc.
#[allow(clippy::type_complexity)]
pub trait AppService: Send + Sync + 'static {
    /// Add creates a new App
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn add<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get retrieves a App
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn get<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, Self>,
        >,
    > + Send;
    /// Patch updates an existing App
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn patch<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, Self>,
        >,
    > + Send;
    /// Apply applies a patch document to an existing App
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn apply<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<crate::proto::wafl::App> + Send + use<'a, Self>,
        >,
    > + Send;
    /// Erase deletes a App
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn erase<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppEraseResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List reads Apps a page at a time.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn list<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AppListResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Stream one execution through the controller; no runner network access or
    /// Kubernetes port-forward is required. Omitted instance selects a ready
    /// execution once. The response metadata includes its wafl-instance UUID.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn tail(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AppTailRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<crate::proto::wafl::Log> + Send + use<Self>,
            >,
        >,
    > + Send;
}
/// Extension trait for registering a service implementation with a Router.
///
/// This trait is automatically implemented for all types that implement the service trait.
/// Prefer [`Router::add_service`](::connectrpc::Router::add_service) for
/// top-down registration; `register` remains available for compatibility
/// and cases where the service-first call shape is more convenient.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
///
/// let service = Arc::new(MyServiceImpl);
/// let router = service.register(Router::new());
/// ```
pub trait AppServiceExt: AppService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: AppService> AppServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "Add",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppAddRequestView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppAddRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::App>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_ADD_SPEC)
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "Get",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppGetRequestView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppGetRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::App>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_GET_SPEC)
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "Patch",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppPatchRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppPatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.patch(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::App>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_PATCH_SPEC)
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "Apply",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppApplyRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppApplyRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.apply(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::App>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_APPLY_SPEC)
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "Erase",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppRefView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppRef,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.erase(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::AppEraseResponse>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_ERASE_SPEC)
            .route_view(
                APP_SERVICE_SERVICE_NAME,
                "List",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppListRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppListRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::AppListResponse>(format)
                        }
                    })
                },
            )
            .with_spec(APP_SERVICE_LIST_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::proto::wafl::Log,
            >(
                APP_SERVICE_SERVICE_NAME,
                "Tail",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AppTailRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AppTailRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.tail(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(APP_SERVICE_TAIL_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct AppServiceRegisterMarker;
impl<S: AppService> ::connectrpc::ServiceRegister<AppServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as AppServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `AppService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = AppServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct AppServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: AppService> AppServiceServer<T> {
    /// Wrap a service implementation in a monomorphic dispatcher.
    pub fn new(service: T) -> Self {
        Self {
            inner: ::std::sync::Arc::new(service),
        }
    }
    /// Wrap an already-`Arc`'d service implementation.
    pub fn from_arc(inner: ::std::sync::Arc<T>) -> Self {
        Self { inner }
    }
}
impl<T> Clone for AppServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: AppService> ::connectrpc::Dispatcher for AppServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("wafl.AppService/")?;
        match method {
            "Add" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_ADD_SPEC),
                )
            }
            "Get" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_GET_SPEC),
                )
            }
            "Patch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_PATCH_SPEC),
                )
            }
            "Apply" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_APPLY_SPEC),
                )
            }
            "Erase" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_ERASE_SPEC),
                )
            }
            "List" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(APP_SERVICE_LIST_SPEC),
                )
            }
            "Tail" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(APP_SERVICE_TAIL_SPEC),
                )
            }
            _ => None,
        }
    }
    fn call_unary(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::Payload,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("wafl.AppService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Add" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppAddRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppAddRequestView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppAddRequest,
                    >::from_parts(&req, &body);
                    svc.add(ctx, req).await?.encode::<crate::proto::wafl::App>(format)
                })
            }
            "Get" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppGetRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppGetRequestView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppGetRequest,
                    >::from_parts(&req, &body);
                    svc.get(ctx, req).await?.encode::<crate::proto::wafl::App>(format)
                })
            }
            "Patch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppPatchRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppPatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppPatchRequest,
                    >::from_parts(&req, &body);
                    svc.patch(ctx, req).await?.encode::<crate::proto::wafl::App>(format)
                })
            }
            "Apply" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppApplyRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppApplyRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppApplyRequest,
                    >::from_parts(&req, &body);
                    svc.apply(ctx, req).await?.encode::<crate::proto::wafl::App>(format)
                })
            }
            "Erase" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppRef,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppRefView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppRef,
                    >::from_parts(&req, &body);
                    svc.erase(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::AppEraseResponse>(format)
                })
            }
            "List" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppListRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppListRequestView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppListRequest,
                    >::from_parts(&req, &body);
                    svc.list(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::AppListResponse>(format)
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_server_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        request: ::buffa::bytes::Bytes,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("wafl.AppService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Tail" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AppTailRequest,
                    >(request, format)?;
                    let req: crate::proto::wafl::__buffa::view::AppTailRequestView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AppTailRequest,
                    >::from_parts(&req, &body);
                    let resp = svc.tail(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::proto::wafl::Log,
                                _,
                                _,
                            >(s, format)),
                    )
                })
            }
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
    fn call_client_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::UnaryResult {
        let Some(method) = path.strip_prefix("wafl.AppService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_unary(path),
        }
    }
    fn call_bidi_streaming(
        &self,
        path: &str,
        ctx: ::connectrpc::RequestContext,
        requests: ::connectrpc::dispatcher::codegen::RequestStream,
        format: ::connectrpc::CodecFormat,
    ) -> ::connectrpc::dispatcher::codegen::StreamingResult {
        let Some(method) = path.strip_prefix("wafl.AppService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &requests, &format);
        match method {
            _ => ::connectrpc::dispatcher::codegen::unimplemented_streaming(path),
        }
    }
}
/// Client for this service.
///
/// Generic over `T: ClientTransport`. For **gRPC** (HTTP/2), use
/// `Http2Connection` — it has honest `poll_ready` and composes with
/// `tower::balance` for multi-connection load balancing. For **Connect
/// over HTTP/1.1** (or unknown protocol), use `HttpClient`.
///
/// # Example (gRPC / HTTP/2)
///
/// ```rust,ignore
/// use connectrpc::client::{Http2Connection, ClientConfig};
/// use connectrpc::Protocol;
///
/// let uri: http::Uri = "http://localhost:8080".parse()?;
/// let conn = Http2Connection::connect_plaintext(uri.clone()).await?.shared(1024);
/// let config = ClientConfig::new(uri).with_protocol(Protocol::Grpc);
///
/// let client = AppServiceClient::new(conn, config);
/// let response = client.add(request).await?;
/// ```
///
/// # Example (Connect / HTTP/1.1 or ALPN)
///
/// ```rust,ignore
/// use connectrpc::client::{HttpClient, ClientConfig};
///
/// let http = HttpClient::plaintext();  // cleartext http:// only
/// let config = ClientConfig::new("http://localhost:8080".parse()?);
///
/// let client = AppServiceClient::new(http, config);
/// let response = client.add(request).await?;
/// ```
///
/// # Working with the response
///
/// Unary calls return [`UnaryResponse<OwnedView<FooView>>`](::connectrpc::client::UnaryResponse).
/// [`view()`](::connectrpc::client::UnaryResponse::view) borrows the response
/// message, so field access is zero-copy:
///
/// ```rust,ignore
/// let resp = client.add(request).await?;
/// let name: &str = resp.view().name;  // borrow into the response buffer
/// ```
///
/// If you need the owned struct (e.g. to store or pass by value), use
/// [`into_owned()`](::connectrpc::client::UnaryResponse::into_owned):
///
/// ```rust,ignore
/// let owned = client.add(request).await?.into_owned();
/// ```
///
/// [`into_view()`](::connectrpc::client::UnaryResponse::into_view) keeps the
/// zero-copy decoded body (an `OwnedView`) without copying; field access on it
/// goes through `.reborrow()`. Streaming responses yield one
/// [`StreamMessage`](::connectrpc::StreamMessage) per received message from
/// `.message().await` — read fields zero-copy through the generated accessor
/// methods (`msg.name()`) or `.view()`, or convert with `.to_owned_message()`.
#[cfg(feature = "client")]
#[derive(Clone)]
pub struct AppServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> AppServiceClient<T>
where
    T: ::connectrpc::client::ClientTransport,
    <T::ResponseBody as ::connectrpc::http_body::Body>::Error: ::std::fmt::Display,
{
    /// Create a new client with the given transport and configuration.
    pub fn new(transport: T, config: ::connectrpc::client::ClientConfig) -> Self {
        Self { transport, config }
    }
    /// Get the client configuration.
    pub fn config(&self) -> &::connectrpc::client::ClientConfig {
        &self.config
    }
    /// Get a mutable reference to the client configuration.
    pub fn config_mut(&mut self) -> &mut ::connectrpc::client::ClientConfig {
        &mut self.config
    }
    /// Call the Add RPC. Sends a request to /wafl.AppService/Add.
    pub async fn add(
        &self,
        request: crate::proto::wafl::AppAddRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.add_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Add RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn add_with_options(
        &self,
        request: crate::proto::wafl::AppAddRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Add",
                request,
                options,
            )
            .await
    }
    /// Call the Get RPC. Sends a request to /wafl.AppService/Get.
    pub async fn get(
        &self,
        request: crate::proto::wafl::AppGetRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.get_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Get RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn get_with_options(
        &self,
        request: crate::proto::wafl::AppGetRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Get",
                request,
                options,
            )
            .await
    }
    /// Call the Patch RPC. Sends a request to /wafl.AppService/Patch.
    pub async fn patch(
        &self,
        request: crate::proto::wafl::AppPatchRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.patch_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Patch RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn patch_with_options(
        &self,
        request: crate::proto::wafl::AppPatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Patch",
                request,
                options,
            )
            .await
    }
    /// Call the Apply RPC. Sends a request to /wafl.AppService/Apply.
    pub async fn apply(
        &self,
        request: crate::proto::wafl::AppApplyRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.apply_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Apply RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn apply_with_options(
        &self,
        request: crate::proto::wafl::AppApplyRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AppView<'static>>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Apply",
                request,
                options,
            )
            .await
    }
    /// Call the Erase RPC. Sends a request to /wafl.AppService/Erase.
    pub async fn erase(
        &self,
        request: crate::proto::wafl::AppRef,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AppEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.erase_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Erase RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn erase_with_options(
        &self,
        request: crate::proto::wafl::AppRef,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AppEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Erase",
                request,
                options,
            )
            .await
    }
    /// Call the List RPC. Sends a request to /wafl.AppService/List.
    pub async fn list(
        &self,
        request: crate::proto::wafl::AppListRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AppListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.list_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the List RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn list_with_options(
        &self,
        request: crate::proto::wafl::AppListRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AppListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "List",
                request,
                options,
            )
            .await
    }
    /// Call the Tail RPC. Sends a request to /wafl.AppService/Tail.
    pub async fn tail(
        &self,
        request: crate::proto::wafl::AppTailRequest,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::wafl::__buffa::view::LogView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.tail_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Tail RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn tail_with_options(
        &self,
        request: crate::proto::wafl::AppTailRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::wafl::__buffa::view::LogView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                APP_SERVICE_SERVICE_NAME,
                "Tail",
                request,
                options,
            )
            .await
    }
}
include!("wafl.app_svc.g.__layers.rs");
