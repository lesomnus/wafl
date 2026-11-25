///Shorthand for `OwnedView<InstanceAddRequestView<'static>>`.
pub type OwnedInstanceAddRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceAddRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceView<'static>>`.
pub type OwnedInstanceView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceView<'static>,
>;
///Shorthand for `OwnedView<InstanceGetRequestView<'static>>`.
pub type OwnedInstanceGetRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceGetRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceRefView<'static>>`.
pub type OwnedInstanceRefView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceRefView<'static>,
>;
///Shorthand for `OwnedView<InstanceEraseResponseView<'static>>`.
pub type OwnedInstanceEraseResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceEraseResponseView<'static>,
>;
///Shorthand for `OwnedView<InstanceListRequestView<'static>>`.
pub type OwnedInstanceListRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceListRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceListResponseView<'static>>`.
pub type OwnedInstanceListResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceListResponseView<'static>,
>;
///Shorthand for `OwnedView<InstanceStartRequestView<'static>>`.
pub type OwnedInstanceStartRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceStartRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceStopRequestView<'static>>`.
pub type OwnedInstanceStopRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceStopRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceDrainRequestView<'static>>`.
pub type OwnedInstanceDrainRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceDrainRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceReleaseRequestView<'static>>`.
pub type OwnedInstanceReleaseRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceReleaseRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceSealRequestView<'static>>`.
pub type OwnedInstanceSealRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceSealRequestView<'static>,
>;
///Shorthand for `OwnedView<InstanceSealResponseView<'static>>`.
pub type OwnedInstanceSealResponseView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceSealResponseView<'static>,
>;
///Shorthand for `OwnedView<InstanceTailRequestView<'static>>`.
pub type OwnedInstanceTailRequestView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceTailRequestView<'static>,
>;
///Shorthand for `OwnedView<LogView<'static>>`.
pub type OwnedLogView = ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::LogView<'static>,
>;
impl ::connectrpc::Encodable<crate::proto::wafl::Instance>
for crate::proto::wafl::__buffa::view::InstanceView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Instance>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::InstanceView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceEraseResponse>
for crate::proto::wafl::__buffa::view::InstanceEraseResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceEraseResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceEraseResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceListResponse>
for crate::proto::wafl::__buffa::view::InstanceListResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceListResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceListResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceSealResponse>
for crate::proto::wafl::__buffa::view::InstanceSealResponseView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::InstanceSealResponse>
for ::buffa::view::OwnedView<
    crate::proto::wafl::__buffa::view::InstanceSealResponseView<'static>,
> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Log>
for crate::proto::wafl::__buffa::view::LogView<'_> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self, codec)
    }
}
impl ::connectrpc::Encodable<crate::proto::wafl::Log>
for ::buffa::view::OwnedView<crate::proto::wafl::__buffa::view::LogView<'static>> {
    fn encode(
        &self,
        codec: ::connectrpc::CodecFormat,
    ) -> ::std::result::Result<::buffa::bytes::Bytes, ::connectrpc::ConnectError> {
        ::connectrpc::__codegen::encode_view_body(self.reborrow(), codec)
    }
}
/// Full service name for this service.
pub const INSTANCE_SERVICE_SERVICE_NAME: &str = "wafl.InstanceService";
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Add` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_ADD_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Add",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Get` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_GET_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Get",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Erase` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_ERASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Erase",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `List` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_LIST_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/List",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Start` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_START_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Start",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Stop` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_STOP_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Stop",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Drain` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_DRAIN_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Drain",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Release` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_RELEASE_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Release",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Seal` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_SEAL_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Seal",
        ::connectrpc::StreamType::Unary,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Static [`Spec`](::connectrpc::Spec) for the server-side `Tail` RPC.
///
/// The dispatcher surfaces this on
/// [`RequestContext::spec`](::connectrpc::RequestContext::spec).
pub const INSTANCE_SERVICE_TAIL_SPEC: ::connectrpc::Spec = ::connectrpc::Spec::server(
        "/wafl.InstanceService/Tail",
        ::connectrpc::StreamType::ServerStream,
    )
    .with_idempotency_level(::connectrpc::IdempotencyLevel::Unknown);
/// Server trait for InstanceService.
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
pub trait InstanceService: Send + Sync + 'static {
    /// Add creates a new Instance
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceAddRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Get retrieves a Instance
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceGetRequest>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Erase deletes a Instance
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
        request: ::connectrpc::ServiceRequest<'_, crate::proto::wafl::InstanceRef>,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceEraseResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// List reads Instances a page at a time.
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
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceListRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceListResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Reuse a running instance or wait for the shared startup attempt.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn start<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceStartRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Proxy mode: refuse active requests; a later request starts the instance again.
    /// Direct mode: requires the endpoint reservation to have been Released first.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn stop<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceStopRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Proxy mode: close admission, finish requests, then stop. Start resumes it.
    /// Direct mode: mark draining and return; gateways withdraw before Release.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn drain<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceDrainRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Direct mode only. Acknowledge that ALL gateways have withdrawn this
    /// generation and closed/drained their connections, then stop and free its
    /// endpoint reservation. Requires prior Drain and wafl-session/wafl-revision.
    /// The supervisor completes even if the RPC caller disconnects.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn release<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceReleaseRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::Instance,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Fence all future Add calls for this ID in the current runner session.
    /// Does not interrupt an already published execution; Drain/Release still
    /// apply. Allows safe Erase after release despite delayed controller calls.
    ///
    /// `'a` lets the response body borrow from `&self` (e.g. server-resident state).
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call; message fields are read directly on it
    /// (zero-copy). The response cannot borrow from `request` — use
    /// `.to_owned_message()` (or copy the specific fields) for anything
    /// returned, stored, or moved into `tokio::spawn`.
    fn seal<'a>(
        &'a self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceSealRequest,
        >,
    ) -> impl ::std::future::Future<
        Output = ::connectrpc::ServiceResult<
            impl ::connectrpc::Encodable<
                crate::proto::wafl::InstanceSealResponse,
            > + Send + use<'a, Self>,
        >,
    > + Send;
    /// Read retained stdout/stderr, optionally following across Stop/Start.
    ///
    /// `request` is borrowed from the request body and is valid for the
    /// duration of the call (until the response stream is returned);
    /// message fields are read directly on it (zero-copy). Data the
    /// returned stream needs must be copied out or converted via
    /// `.to_owned_message()`.
    fn tail(
        &self,
        ctx: ::connectrpc::RequestContext,
        request: ::connectrpc::ServiceRequest<
            '_,
            crate::proto::wafl::InstanceTailRequest,
        >,
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
pub trait InstanceServiceExt: InstanceService {
    /// Register this service implementation with a Router.
    ///
    /// Takes ownership of the `Arc<Self>` and returns a new Router with
    /// this service's methods registered.
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router;
}
impl<S: InstanceService> InstanceServiceExt for S {
    fn register(
        self: ::std::sync::Arc<Self>,
        router: ::connectrpc::Router,
    ) -> ::connectrpc::Router {
        router
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Add",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceAddRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceAddRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.add(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_ADD_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Get",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceGetRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceGetRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.get(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_GET_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Erase",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceRefView<'static>,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceRef,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.erase(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::InstanceEraseResponse>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_ERASE_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "List",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceListRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceListRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.list(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::InstanceListResponse>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_LIST_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Start",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceStartRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceStartRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.start(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_START_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Stop",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceStopRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceStopRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.stop(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_STOP_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Drain",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceDrainRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceDrainRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.drain(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_DRAIN_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Release",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceReleaseRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceReleaseRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.release(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::Instance>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_RELEASE_SPEC)
            .route_view(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Seal",
                {
                    let svc = ::std::sync::Arc::clone(&self);
                    ::connectrpc::view_handler_fn(move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceSealRequestView<
                                'static,
                            >,
                        >,
                        format|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceSealRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.seal(ctx, sreq)
                                .await?
                                .encode::<crate::proto::wafl::InstanceSealResponse>(format)
                        }
                    })
                },
            )
            .with_spec(INSTANCE_SERVICE_SEAL_SPEC)
            .route_view_server_stream::<
                _,
                _,
                crate::proto::wafl::Log,
            >(
                INSTANCE_SERVICE_SERVICE_NAME,
                "Tail",
                ::connectrpc::view_streaming_handler_fn({
                    let svc = ::std::sync::Arc::clone(&self);
                    move |
                        ctx,
                        req: ::buffa::view::OwnedView<
                            crate::proto::wafl::__buffa::view::InstanceTailRequestView<
                                'static,
                            >,
                        >|
                    {
                        let svc = ::std::sync::Arc::clone(&svc);
                        async move {
                            let sreq = ::connectrpc::ServiceRequest::<
                                crate::proto::wafl::InstanceTailRequest,
                            >::from_parts(req.reborrow(), req.bytes());
                            svc.tail(ctx, sreq).await
                        }
                    }
                }),
            )
            .with_spec(INSTANCE_SERVICE_TAIL_SPEC)
    }
}
/// Type-inference marker used by [`Router::add_service`](::connectrpc::Router::add_service).
#[doc(hidden)]
pub struct InstanceServiceRegisterMarker;
impl<S: InstanceService> ::connectrpc::ServiceRegister<InstanceServiceRegisterMarker>
for ::std::sync::Arc<S> {
    fn register_service(self, router: ::connectrpc::Router) -> ::connectrpc::Router {
        <S as InstanceServiceExt>::register(self, router)
    }
}
/// Monomorphic dispatcher for `InstanceService`.
///
/// Unlike `.register(Router)` which type-erases each method into an `Arc<dyn ErasedHandler>` stored in a `HashMap`, this struct dispatches via a compile-time `match` on method name: no vtable, no hash lookup.
///
/// # Example
///
/// ```rust,ignore
/// use connectrpc::ConnectRpcService;
///
/// let server = InstanceServiceServer::new(MyImpl);
/// let service = ConnectRpcService::new(server);
/// // hand `service` to axum/hyper as a fallback_service
/// ```
pub struct InstanceServiceServer<T> {
    inner: ::std::sync::Arc<T>,
}
impl<T: InstanceService> InstanceServiceServer<T> {
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
impl<T> Clone for InstanceServiceServer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ::std::sync::Arc::clone(&self.inner),
        }
    }
}
impl<T: InstanceService> ::connectrpc::Dispatcher for InstanceServiceServer<T> {
    #[inline]
    fn lookup(
        &self,
        path: &str,
    ) -> Option<::connectrpc::dispatcher::codegen::MethodDescriptor> {
        let method = path.strip_prefix("wafl.InstanceService/")?;
        match method {
            "Add" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_ADD_SPEC),
                )
            }
            "Get" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_GET_SPEC),
                )
            }
            "Erase" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_ERASE_SPEC),
                )
            }
            "List" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_LIST_SPEC),
                )
            }
            "Start" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_START_SPEC),
                )
            }
            "Stop" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_STOP_SPEC),
                )
            }
            "Drain" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_DRAIN_SPEC),
                )
            }
            "Release" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_RELEASE_SPEC),
                )
            }
            "Seal" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::unary(false)
                        .with_spec(INSTANCE_SERVICE_SEAL_SPEC),
                )
            }
            "Tail" => {
                Some(
                    ::connectrpc::dispatcher::codegen::MethodDescriptor::server_streaming()
                        .with_spec(INSTANCE_SERVICE_TAIL_SPEC),
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
        let Some(method) = path.strip_prefix("wafl.InstanceService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_unary(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Add" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceAddRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceAddRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceAddRequest,
                    >::from_parts(&req, &body);
                    svc.add(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Get" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceGetRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceGetRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceGetRequest,
                    >::from_parts(&req, &body);
                    svc.get(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Erase" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceRef,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceRefView<'_> = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceRef,
                    >::from_parts(&req, &body);
                    svc.erase(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::InstanceEraseResponse>(format)
                })
            }
            "List" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceListRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceListRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceListRequest,
                    >::from_parts(&req, &body);
                    svc.list(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::InstanceListResponse>(format)
                })
            }
            "Start" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceStartRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceStartRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceStartRequest,
                    >::from_parts(&req, &body);
                    svc.start(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Stop" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceStopRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceStopRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceStopRequest,
                    >::from_parts(&req, &body);
                    svc.stop(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Drain" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceDrainRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceDrainRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceDrainRequest,
                    >::from_parts(&req, &body);
                    svc.drain(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Release" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceReleaseRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceReleaseRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceReleaseRequest,
                    >::from_parts(&req, &body);
                    svc.release(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::Instance>(format)
                })
            }
            "Seal" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceSealRequest,
                    >(request.encoded()?, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceSealRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceSealRequest,
                    >::from_parts(&req, &body);
                    svc.seal(ctx, req)
                        .await?
                        .encode::<crate::proto::wafl::InstanceSealResponse>(format)
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
        let Some(method) = path.strip_prefix("wafl.InstanceService/") else {
            return ::connectrpc::dispatcher::codegen::unimplemented_streaming(path);
        };
        let _ = (&ctx, &request, &format);
        match method {
            "Tail" => {
                let svc = ::std::sync::Arc::clone(&self.inner);
                Box::pin(async move {
                    let body = ::connectrpc::dispatcher::codegen::request_proto_bytes::<
                        crate::proto::wafl::InstanceTailRequest,
                    >(request, format)?;
                    let req: crate::proto::wafl::__buffa::view::InstanceTailRequestView<
                        '_,
                    > = ::connectrpc::dispatcher::codegen::decode_borrowed_request_view(
                        &body,
                    )?;
                    let req = ::connectrpc::ServiceRequest::<
                        crate::proto::wafl::InstanceTailRequest,
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
        let Some(method) = path.strip_prefix("wafl.InstanceService/") else {
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
        let Some(method) = path.strip_prefix("wafl.InstanceService/") else {
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
/// let client = InstanceServiceClient::new(conn, config);
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
/// let client = InstanceServiceClient::new(http, config);
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
pub struct InstanceServiceClient<T> {
    transport: T,
    config: ::connectrpc::client::ClientConfig,
}
#[cfg(feature = "client")]
impl<T> InstanceServiceClient<T>
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
    /// Call the Add RPC. Sends a request to /wafl.InstanceService/Add.
    pub async fn add(
        &self,
        request: crate::proto::wafl::InstanceAddRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
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
        request: crate::proto::wafl::InstanceAddRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Add",
                request,
                options,
            )
            .await
    }
    /// Call the Get RPC. Sends a request to /wafl.InstanceService/Get.
    pub async fn get(
        &self,
        request: crate::proto::wafl::InstanceGetRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
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
        request: crate::proto::wafl::InstanceGetRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Get",
                request,
                options,
            )
            .await
    }
    /// Call the Erase RPC. Sends a request to /wafl.InstanceService/Erase.
    pub async fn erase(
        &self,
        request: crate::proto::wafl::InstanceRef,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceEraseResponseView<'static>,
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
        request: crate::proto::wafl::InstanceRef,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceEraseResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Erase",
                request,
                options,
            )
            .await
    }
    /// Call the List RPC. Sends a request to /wafl.InstanceService/List.
    pub async fn list(
        &self,
        request: crate::proto::wafl::InstanceListRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceListResponseView<'static>,
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
        request: crate::proto::wafl::InstanceListRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceListResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "List",
                request,
                options,
            )
            .await
    }
    /// Call the Start RPC. Sends a request to /wafl.InstanceService/Start.
    pub async fn start(
        &self,
        request: crate::proto::wafl::InstanceStartRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.start_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Start RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn start_with_options(
        &self,
        request: crate::proto::wafl::InstanceStartRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Start",
                request,
                options,
            )
            .await
    }
    /// Call the Stop RPC. Sends a request to /wafl.InstanceService/Stop.
    pub async fn stop(
        &self,
        request: crate::proto::wafl::InstanceStopRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.stop_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Stop RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn stop_with_options(
        &self,
        request: crate::proto::wafl::InstanceStopRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Stop",
                request,
                options,
            )
            .await
    }
    /// Call the Drain RPC. Sends a request to /wafl.InstanceService/Drain.
    pub async fn drain(
        &self,
        request: crate::proto::wafl::InstanceDrainRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.drain_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Drain RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn drain_with_options(
        &self,
        request: crate::proto::wafl::InstanceDrainRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Drain",
                request,
                options,
            )
            .await
    }
    /// Call the Release RPC. Sends a request to /wafl.InstanceService/Release.
    pub async fn release(
        &self,
        request: crate::proto::wafl::InstanceReleaseRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.release_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Release RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn release_with_options(
        &self,
        request: crate::proto::wafl::InstanceReleaseRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Release",
                request,
                options,
            )
            .await
    }
    /// Call the Seal RPC. Sends a request to /wafl.InstanceService/Seal.
    pub async fn seal(
        &self,
        request: crate::proto::wafl::InstanceSealRequest,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceSealResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        self.seal_with_options(request, ::connectrpc::client::CallOptions::default())
            .await
    }
    /// Call the Seal RPC with explicit per-call options. Options override [`ClientConfig`](::connectrpc::client::ClientConfig) defaults.
    pub async fn seal_with_options(
        &self,
        request: crate::proto::wafl::InstanceSealRequest,
        options: ::connectrpc::client::CallOptions,
    ) -> Result<
        ::connectrpc::client::UnaryResponse<
            ::buffa::view::OwnedView<
                crate::proto::wafl::__buffa::view::InstanceSealResponseView<'static>,
            >,
        >,
        ::connectrpc::ConnectError,
    > {
        ::connectrpc::client::call_unary(
                &self.transport,
                &self.config,
                INSTANCE_SERVICE_SERVICE_NAME,
                "Seal",
                request,
                options,
            )
            .await
    }
    /// Call the Tail RPC. Sends a request to /wafl.InstanceService/Tail.
    pub async fn tail(
        &self,
        request: crate::proto::wafl::InstanceTailRequest,
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
        request: crate::proto::wafl::InstanceTailRequest,
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
                INSTANCE_SERVICE_SERVICE_NAME,
                "Tail",
                request,
                options,
            )
            .await
    }
}
include!("wafl.instance_svc.g.__layers.rs");
