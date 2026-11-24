///Shorthand for `OwnedView<AuditAddRequestView<'static>>`.
pub type OwnedAuditAddRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditAddRequestView<'static>,
>;
///Shorthand for `OwnedView<AuditView<'static>>`.
pub type OwnedAuditView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditView<'static>,
>;
///Shorthand for `OwnedView<AuditGetRequestView<'static>>`.
pub type OwnedAuditGetRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditGetRequestView<'static>,
>;
///Shorthand for `OwnedView<AuditPatchRequestView<'static>>`.
pub type OwnedAuditPatchRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditPatchRequestView<'static>,
>;
///Shorthand for `OwnedView<AuditApplyRequestView<'static>>`.
pub type OwnedAuditApplyRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditApplyRequestView<'static>,
>;
///Shorthand for `OwnedView<AuditRefView<'static>>`.
pub type OwnedAuditRefView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditRefView<'static>,
>;
///Shorthand for `OwnedView<AuditEraseResponseView<'static>>`.
pub type OwnedAuditEraseResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditEraseResponseView<'static>,
>;
///Shorthand for `OwnedView<AuditListRequestView<'static>>`.
pub type OwnedAuditListRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditListRequestView<'static>,
>;
///Shorthand for `OwnedView<AuditListResponseView<'static>>`.
pub type OwnedAuditListResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditListResponseView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::wafl::Audit>
for crate::proto::wafl::__buffa::view::AuditView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Audit>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::AuditView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AuditEraseResponse>
for crate::proto::wafl::__buffa::view::AuditEraseResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AuditEraseResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditEraseResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AuditListResponse>
for crate::proto::wafl::__buffa::view::AuditListResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::AuditListResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::AuditListResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const AUDIT_SERVICE_SERVICE_NAME: &str = "wafl.AuditService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Add` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_ADD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/Add",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Get` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_GET_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/Get",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Patch` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_PATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/Patch",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Apply` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_APPLY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/Apply",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Erase` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_ERASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/Erase",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `List` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const AUDIT_SERVICE_LIST_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.AuditService/List",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for AuditService.
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
pub trait AuditService: Send + Sync + 'static {
    /// Add creates a new Audit
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Audit,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get retrieves a Audit
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Audit,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Patch updates an existing Audit
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Audit,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Apply applies a patch document to an existing Audit
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Audit,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Erase deletes a Audit
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AuditEraseResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List reads Audits a page at a time.
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::AuditListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::AuditListResponse,
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
pub trait AuditServiceExt: AuditService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: AuditService> AuditServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "Add",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditAddRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditAddRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Audit>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_ADD_SPEC)
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "Get",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditGetRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditGetRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Audit>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_GET_SPEC)
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "Patch",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditPatchRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditPatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.patch(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Audit>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_PATCH_SPEC)
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "Apply",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditApplyRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditApplyRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.apply(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Audit>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_APPLY_SPEC)
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "Erase",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditRefView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditRef,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.erase(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::AuditEraseResponse>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_ERASE_SPEC)
            .route_view(
                AUDIT_SERVICE_SERVICE_NAME,
                "List",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::AuditListRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::AuditListRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::AuditListResponse>(format)
                        }
                    })
                },
            )
            .with_spec(AUDIT_SERVICE_LIST_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct AuditServiceRegisterMarker;
impl<S: AuditService> ::connectrpc::ServiceRegister<AuditServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as AuditServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `AuditService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = AuditServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct AuditServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: AuditService> AuditServiceServer<T> {
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
impl<T> Clone for AuditServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: AuditService> ::connectrpc::Dispatcher for AuditServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("wafl.AuditService/")?;
        match method {
            "Add" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_ADD_SPEC),
                )
            }
            "Get" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_GET_SPEC),
                )
            }
            "Patch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_PATCH_SPEC),
                )
            }
            "Apply" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_APPLY_SPEC),
                )
            }
            "Erase" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_ERASE_SPEC),
                )
            }
            "List" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(AUDIT_SERVICE_LIST_SPEC),
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
        let Some(method) = path.strip_prefix("wafl.AuditService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Add" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditAddRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditAddRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditAddRequest,
                    >::from_parts(&req, &body);
                    svc.add(ctx, req).await?.encode::<crate::proto::wafl::Audit>(format)
                })
            }
            "Get" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditGetRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditGetRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditGetRequest,
                    >::from_parts(&req, &body);
                    svc.get(ctx, req).await?.encode::<crate::proto::wafl::Audit>(format)
                })
            }
            "Patch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditPatchRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditPatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditPatchRequest,
                    >::from_parts(&req, &body);
                    svc.patch(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Audit>(format)
                })
            }
            "Apply" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditApplyRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditApplyRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditApplyRequest,
                    >::from_parts(&req, &body);
                    svc.apply(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Audit>(format)
                })
            }
            "Erase" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditRef,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditRefView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditRef,
                    >::from_parts(&req, &body);
                    svc.erase(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::AuditEraseResponse>(format)
                })
            }
            "List" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::AuditListRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::AuditListRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::AuditListRequest,
                    >::from_parts(&req, &body);
                    svc.list(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::AuditListResponse>(format)
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
        let Some(method) = path.strip_prefix("wafl.AuditService/") else {
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
        let Some(method) = path.strip_prefix("wafl.AuditService/") else {
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
        let Some(method) = path.strip_prefix("wafl.AuditService/") else {
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
/// let client = AuditServiceClient::new(conn, config);
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
/// let client = AuditServiceClient::new(http, config);
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
pub struct AuditServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> AuditServiceClient<T>
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
    /// Call the Add RPC. Sends a request to /wafl.AuditService/Add.
    pub async fn add(
        &self,
        request: crate::proto::wafl::AuditAddRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
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
        request: crate::proto::wafl::AuditAddRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "Add",
                request,
                options,
            )
            .await
    }
    /// Call the Get RPC. Sends a request to /wafl.AuditService/Get.
    pub async fn get(
        &self,
        request: crate::proto::wafl::AuditGetRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
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
        request: crate::proto::wafl::AuditGetRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "Get",
                request,
                options,
            )
            .await
    }
    /// Call the Patch RPC. Sends a request to /wafl.AuditService/Patch.
    pub async fn patch(
        &self,
        request: crate::proto::wafl::AuditPatchRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
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
        request: crate::proto::wafl::AuditPatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "Patch",
                request,
                options,
            )
            .await
    }
    /// Call the Apply RPC. Sends a request to /wafl.AuditService/Apply.
    pub async fn apply(
        &self,
        request: crate::proto::wafl::AuditApplyRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
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
        request: crate::proto::wafl::AuditApplyRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "Apply",
                request,
                options,
            )
            .await
    }
    /// Call the Erase RPC. Sends a request to /wafl.AuditService/Erase.
    pub async fn erase(
        &self,
        request: crate::proto::wafl::AuditRef,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditEraseResponseView<'static>,
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
        request: crate::proto::wafl::AuditRef,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "Erase",
                request,
                options,
            )
            .await
    }
    /// Call the List RPC. Sends a request to /wafl.AuditService/List.
    pub async fn list(
        &self,
        request: crate::proto::wafl::AuditListRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditListResponseView<'static>,
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
        request: crate::proto::wafl::AuditListRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::AuditListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                AUDIT_SERVICE_SERVICE_NAME,
                "List",
                request,
                options,
            )
            .await
    }
}
include!("wafl.payday.audit_svc.g.__layers.rs");
