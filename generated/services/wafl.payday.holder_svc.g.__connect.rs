///Shorthand for `OwnedView<HolderAddRequestView<'static>>`.
pub type OwnedHolderAddRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderAddRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderView<'static>>`.
pub type OwnedHolderView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderView<'static>,
>;
///Shorthand for `OwnedView<HolderGetRequestView<'static>>`.
pub type OwnedHolderGetRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderGetRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderPatchRequestView<'static>>`.
pub type OwnedHolderPatchRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderPatchRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderApplyRequestView<'static>>`.
pub type OwnedHolderApplyRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderApplyRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderRefView<'static>>`.
pub type OwnedHolderRefView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderRefView<'static>,
>;
///Shorthand for `OwnedView<HolderEraseResponseView<'static>>`.
pub type OwnedHolderEraseResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderEraseResponseView<'static>,
>;
///Shorthand for `OwnedView<HolderListRequestView<'static>>`.
pub type OwnedHolderListRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderListRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderListResponseView<'static>>`.
pub type OwnedHolderListResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderListResponseView<'static>,
>;
///Shorthand for `OwnedView<HolderWatchRequestView<'static>>`.
pub type OwnedHolderWatchRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderWatchRequestView<'static>,
>;
///Shorthand for `OwnedView<HolderWatchResponseView<'static>>`.
pub type OwnedHolderWatchResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderWatchResponseView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::wafl::Holder>
for crate::proto::wafl::__buffa::view::HolderView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Holder>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::HolderView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderEraseResponse>
for crate::proto::wafl::__buffa::view::HolderEraseResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderEraseResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderEraseResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderListResponse>
for crate::proto::wafl::__buffa::view::HolderListResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderListResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderListResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderWatchResponse>
for crate::proto::wafl::__buffa::view::HolderWatchResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::HolderWatchResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::HolderWatchResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const HOLDER_SERVICE_SERVICE_NAME: &str = "wafl.HolderService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Add` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_ADD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Add",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Get` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_GET_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Get",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Patch` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_PATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Patch",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Apply` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_APPLY_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Apply",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Erase` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_ERASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Erase",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `List` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_LIST_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/List",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Watch` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const HOLDER_SERVICE_WATCH_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.HolderService/Watch",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for HolderService.
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
pub trait HolderService: Send + Sync + 'static {
    /// Add creates a new Holder
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Holder,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get retrieves a Holder
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Holder,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Patch updates an existing Holder
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderPatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Holder,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Apply applies a patch document to an existing Holder
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderApplyRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Holder,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Erase deletes a Holder
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::HolderEraseResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List reads Holders a page at a time.
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderListRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::HolderListResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Watch the Holders this caller may see, as they are now and as they change.
    /// What arrives is **state and never a delta**, so a client converges rather
    /// than replays: it keeps what it was last told about a row and replaces it.
    /// An event it did not receive is one it does not need, since the next one
    /// about that row carries the whole of it.
    /// The first message is everything that matches right now, so a client does
    /// not have to List first and race the subscription. A row may arrive twice --
    /// once in that first message and once as a change that happened while it was
    /// being read -- and that is harmless for the same reason.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn watch(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::HolderWatchRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            ::connectrpc::ServiceStream<
                impl ::connectrpc::Encodable<
                    crate::proto::wafl::HolderWatchResponse,
                > + Send + use<Self>,
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
pub trait HolderServiceExt: HolderService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: HolderService> HolderServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "Add",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderAddRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderAddRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Holder>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_ADD_SPEC)
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "Get",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderGetRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderGetRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Holder>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_GET_SPEC)
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "Patch",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderPatchRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderPatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.patch(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Holder>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_PATCH_SPEC)
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "Apply",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderApplyRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderApplyRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.apply(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Holder>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_APPLY_SPEC)
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "Erase",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderRefView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderRef,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.erase(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::HolderEraseResponse>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_ERASE_SPEC)
            .route_view(
                HOLDER_SERVICE_SERVICE_NAME,
                "List",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderListRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderListRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::HolderListResponse>(format)
                        }
                    })
                },
            )
            .with_spec(HOLDER_SERVICE_LIST_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::proto::wafl::HolderWatchResponse,
            >(
                HOLDER_SERVICE_SERVICE_NAME,
                "Watch",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::HolderWatchRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::HolderWatchRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.watch(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(HOLDER_SERVICE_WATCH_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct HolderServiceRegisterMarker;
impl<S: HolderService> ::connectrpc::ServiceRegister<HolderServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as HolderServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `HolderService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = HolderServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct HolderServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: HolderService> HolderServiceServer<T> {
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
impl<T> Clone for HolderServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: HolderService> ::connectrpc::Dispatcher for HolderServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("wafl.HolderService/")?;
        match method {
            "Add" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_ADD_SPEC),
                )
            }
            "Get" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_GET_SPEC),
                )
            }
            "Patch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_PATCH_SPEC),
                )
            }
            "Apply" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_APPLY_SPEC),
                )
            }
            "Erase" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_ERASE_SPEC),
                )
            }
            "List" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(HOLDER_SERVICE_LIST_SPEC),
                )
            }
            "Watch" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(HOLDER_SERVICE_WATCH_SPEC),
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
        let Some(method) = path.strip_prefix("wafl.HolderService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Add" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderAddRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderAddRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderAddRequest,
                    >::from_parts(&req, &body);
                    svc.add(ctx, req).await?.encode::<crate::proto::wafl::Holder>(format)
                })
            }
            "Get" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderGetRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderGetRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderGetRequest,
                    >::from_parts(&req, &body);
                    svc.get(ctx, req).await?.encode::<crate::proto::wafl::Holder>(format)
                })
            }
            "Patch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderPatchRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderPatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderPatchRequest,
                    >::from_parts(&req, &body);
                    svc.patch(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Holder>(format)
                })
            }
            "Apply" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderApplyRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderApplyRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderApplyRequest,
                    >::from_parts(&req, &body);
                    svc.apply(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Holder>(format)
                })
            }
            "Erase" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderRef,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderRefView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderRef,
                    >::from_parts(&req, &body);
                    svc.erase(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::HolderEraseResponse>(format)
                })
            }
            "List" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderListRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderListRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderListRequest,
                    >::from_parts(&req, &body);
                    svc.list(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::HolderListResponse>(format)
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
        let Some(method) = path.strip_prefix("wafl.HolderService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Watch" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::HolderWatchRequest,
                    >(request, format)?;
                    let req: crate::proto::wafl::__buffa::view::HolderWatchRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::HolderWatchRequest,
                    >::from_parts(&req, &body);
                    let resp = svc.watch(ctx, req).await?;
                    Ok(
                        resp
                            .map_body(|s| ::connectrpc::dispatcher::codegen::encode_response_stream::<
                                crate::proto::wafl::HolderWatchResponse,
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
        let Some(method) = path.strip_prefix("wafl.HolderService/") else {
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
        let Some(method) = path.strip_prefix("wafl.HolderService/") else {
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
/// let client = HolderServiceClient::new(conn, config);
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
/// let client = HolderServiceClient::new(http, config);
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
pub struct HolderServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> HolderServiceClient<T>
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
    /// Call the Add RPC. Sends a request to /wafl.HolderService/Add.
    pub async fn add(
        &self,
        request: crate::proto::wafl::HolderAddRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
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
        request: crate::proto::wafl::HolderAddRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Add",
                request,
                options,
            )
            .await
    }
    /// Call the Get RPC. Sends a request to /wafl.HolderService/Get.
    pub async fn get(
        &self,
        request: crate::proto::wafl::HolderGetRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
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
        request: crate::proto::wafl::HolderGetRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Get",
                request,
                options,
            )
            .await
    }
    /// Call the Patch RPC. Sends a request to /wafl.HolderService/Patch.
    pub async fn patch(
        &self,
        request: crate::proto::wafl::HolderPatchRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
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
        request: crate::proto::wafl::HolderPatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Patch",
                request,
                options,
            )
            .await
    }
    /// Call the Apply RPC. Sends a request to /wafl.HolderService/Apply.
    pub async fn apply(
        &self,
        request: crate::proto::wafl::HolderApplyRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
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
        request: crate::proto::wafl::HolderApplyRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Apply",
                request,
                options,
            )
            .await
    }
    /// Call the Erase RPC. Sends a request to /wafl.HolderService/Erase.
    pub async fn erase(
        &self,
        request: crate::proto::wafl::HolderRef,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderEraseResponseView<'static>,
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
        request: crate::proto::wafl::HolderRef,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Erase",
                request,
                options,
            )
            .await
    }
    /// Call the List RPC. Sends a request to /wafl.HolderService/List.
    pub async fn list(
        &self,
        request: crate::proto::wafl::HolderListRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderListResponseView<'static>,
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
        request: crate::proto::wafl::HolderListRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::HolderListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "List",
                request,
                options,
            )
            .await
    }
    /// Call the Watch RPC. Sends a request to /wafl.HolderService/Watch.
    pub async fn watch(
        &self,
        request: crate::proto::wafl::HolderWatchRequest,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::wafl::__buffa::view::HolderWatchResponseView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        self.watch_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Watch RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn watch_with_options(
        &self,
        request: crate::proto::wafl::HolderWatchRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::ServerStream<
            T::ResponseBody,
            crate::proto::wafl::__buffa::view::HolderWatchResponseView<'static>,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_server_stream(
                &self.transport,
                &self.config,
                HOLDER_SERVICE_SERVICE_NAME,
                "Watch",
                request,
                options,
            )
            .await
    }
}
include!("wafl.payday.holder_svc.g.__layers.rs");
