///Shorthand for `OwnedView<TenantAddRequestView<'static>>`.
pub type OwnedTenantAddRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantAddRequestView<'static>,
>;
///Shorthand for `OwnedView<TenantView<'static>>`.
pub type OwnedTenantView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantView<'static>,
>;
///Shorthand for `OwnedView<TenantGetRequestView<'static>>`.
pub type OwnedTenantGetRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantGetRequestView<'static>,
>;
///Shorthand for `OwnedView<TenantPatchRequestView<'static>>`.
pub type OwnedTenantPatchRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantPatchRequestView<'static>,
>;
///Shorthand for `OwnedView<TenantApplyRequestView<'static>>`.
pub type OwnedTenantApplyRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantApplyRequestView<'static>,
>;
///Shorthand for `OwnedView<TenantRefView<'static>>`.
pub type OwnedTenantRefView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantRefView<'static>,
>;
///Shorthand for `OwnedView<TenantEraseResponseView<'static>>`.
pub type OwnedTenantEraseResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantEraseResponseView<'static>,
>;
///Shorthand for `OwnedView<TenantListRequestView<'static>>`.
pub type OwnedTenantListRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantListRequestView<'static>,
>;
///Shorthand for `OwnedView<TenantListResponseView<'static>>`.
pub type OwnedTenantListResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantListResponseView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::wafl::Tenant>
for crate::proto::wafl::__buffa::view::TenantView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Tenant>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::TenantView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::TenantEraseResponse>
for crate::proto::wafl::__buffa::view::TenantEraseResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::TenantEraseResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantEraseResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::TenantListResponse>
for crate::proto::wafl::__buffa::view::TenantListResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::TenantListResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::TenantListResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const TENANT_SERVICE_SERVICE_NAME: &str = "wafl.TenantService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Add` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_ADD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/Add",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Get` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_GET_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/Get",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Patch` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_PATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/Patch",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Apply` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_APPLY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/Apply",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Erase` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_ERASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/Erase",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `List` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const TENANT_SERVICE_LIST_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.TenantService/List",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for TenantService.
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
pub trait TenantService: Send + Sync + 'static {
    /// Add creates a new Tenant
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Tenant,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get retrieves a Tenant
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Tenant,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Patch updates an existing Tenant
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Tenant,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Apply applies a patch document to an existing Tenant
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Tenant,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Erase deletes a Tenant
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::TenantEraseResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List reads Tenants a page at a time.
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::TenantListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::TenantListResponse,
            > + Send + use<'a, Self>,
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
pub trait TenantServiceExt: TenantService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: TenantService> TenantServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "Add",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantAddRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantAddRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Tenant>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_ADD_SPEC)
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "Get",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantGetRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantGetRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Tenant>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_GET_SPEC)
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "Patch",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantPatchRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantPatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.patch(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Tenant>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_PATCH_SPEC)
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "Apply",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantApplyRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantApplyRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.apply(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Tenant>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_APPLY_SPEC)
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "Erase",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantRefView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantRef,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.erase(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::TenantEraseResponse>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_ERASE_SPEC)
            .route_view(
                TENANT_SERVICE_SERVICE_NAME,
                "List",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::TenantListRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::TenantListRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::TenantListResponse>(format)
                        }
                    })
                },
            )
            .with_spec(TENANT_SERVICE_LIST_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct TenantServiceRegisterMarker;
impl<S: TenantService> ::connectrpc::ServiceRegister<TenantServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as TenantServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `TenantService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = TenantServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct TenantServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: TenantService> TenantServiceServer<T> {
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
impl<T> Clone for TenantServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: TenantService> ::connectrpc::Dispatcher for TenantServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("wafl.TenantService/")?;
        match method {
            "Add" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_ADD_SPEC),
                )
            }
            "Get" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_GET_SPEC),
                )
            }
            "Patch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_PATCH_SPEC),
                )
            }
            "Apply" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_APPLY_SPEC),
                )
            }
            "Erase" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_ERASE_SPEC),
                )
            }
            "List" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(TENANT_SERVICE_LIST_SPEC),
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
        let Some(method) = path.strip_prefix("wafl.TenantService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Add" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantAddRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantAddRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantAddRequest,
                    >::from_parts(&req, &body);
                    svc.add(ctx, req).await?.encode::<crate::proto::wafl::Tenant>(format)
                })
            }
            "Get" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantGetRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantGetRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantGetRequest,
                    >::from_parts(&req, &body);
                    svc.get(ctx, req).await?.encode::<crate::proto::wafl::Tenant>(format)
                })
            }
            "Patch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantPatchRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantPatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantPatchRequest,
                    >::from_parts(&req, &body);
                    svc.patch(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Tenant>(format)
                })
            }
            "Apply" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantApplyRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantApplyRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantApplyRequest,
                    >::from_parts(&req, &body);
                    svc.apply(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Tenant>(format)
                })
            }
            "Erase" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantRef,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantRefView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantRef,
                    >::from_parts(&req, &body);
                    svc.erase(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::TenantEraseResponse>(format)
                })
            }
            "List" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::TenantListRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::TenantListRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::TenantListRequest,
                    >::from_parts(&req, &body);
                    svc.list(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::TenantListResponse>(format)
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
        let Some(method) = path.strip_prefix("wafl.TenantService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
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
        let Some(method) = path.strip_prefix("wafl.TenantService/") else {
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
        let Some(method) = path.strip_prefix("wafl.TenantService/") else {
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
/// let client = TenantServiceClient::new(conn, config);
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
/// let client = TenantServiceClient::new(http, config);
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
pub struct TenantServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> TenantServiceClient<T>
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
    /// Call the Add RPC. Sends a request to /wafl.TenantService/Add.
    pub async fn add(
        &self,
        request: crate::proto::wafl::TenantAddRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.add_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Add RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn add_with_options(
        &self,
        request: crate::proto::wafl::TenantAddRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "Add",
                request,
                options,
            )
            .await
    }
    /// Call the Get RPC. Sends a request to /wafl.TenantService/Get.
    pub async fn get(
        &self,
        request: crate::proto::wafl::TenantGetRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.get_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Get RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn get_with_options(
        &self,
        request: crate::proto::wafl::TenantGetRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "Get",
                request,
                options,
            )
            .await
    }
    /// Call the Patch RPC. Sends a request to /wafl.TenantService/Patch.
    pub async fn patch(
        &self,
        request: crate::proto::wafl::TenantPatchRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.patch_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Patch RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn patch_with_options(
        &self,
        request: crate::proto::wafl::TenantPatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "Patch",
                request,
                options,
            )
            .await
    }
    /// Call the Apply RPC. Sends a request to /wafl.TenantService/Apply.
    pub async fn apply(
        &self,
        request: crate::proto::wafl::TenantApplyRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.apply_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Apply RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn apply_with_options(
        &self,
        request: crate::proto::wafl::TenantApplyRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "Apply",
                request,
                options,
            )
            .await
    }
    /// Call the Erase RPC. Sends a request to /wafl.TenantService/Erase.
    pub async fn erase(
        &self,
        request: crate::proto::wafl::TenantRef,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantEraseResponseView<'static>,
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
        request: crate::proto::wafl::TenantRef,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "Erase",
                request,
                options,
            )
            .await
    }
    /// Call the List RPC. Sends a request to /wafl.TenantService/List.
    pub async fn list(
        &self,
        request: crate::proto::wafl::TenantListRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantListResponseView<'static>,
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
        request: crate::proto::wafl::TenantListRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::TenantListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                TENANT_SERVICE_SERVICE_NAME,
                "List",
                request,
                options,
            )
            .await
    }
}
include!("wafl.payday.tenant_svc.g.__layers.rs");
