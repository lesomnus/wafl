CREATE TABLE "pd_wafl__Access" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f8" BYTEA NOT NULL,
  "f9" BYTEA NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" TEXT NOT NULL,
  "f12" TEXT NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BYTEA NOT NULL,
  "f17" BYTEA NOT NULL,
  "f19" BIGINT NOT NULL,
  "f20" TEXT NOT NULL,
  "f21" BIGINT NOT NULL
);
CREATE TABLE "pd_wafl__Archived" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f7" TEXT NOT NULL,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" TEXT NOT NULL,
  "f12" BIGINT NOT NULL,
  "f13" TEXT NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BIGINT NOT NULL,
  "f17" BYTEA NOT NULL,
  "f18" BYTEA NOT NULL
);
CREATE TABLE "pd_wafl__Audit" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f8" BYTEA NOT NULL,
  "f9" BYTEA NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" BYTEA NOT NULL,
  "f12" BYTEA NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BYTEA NOT NULL,
  "f17" BYTEA NOT NULL,
  "f18" BYTEA,
  "f19" BIGINT NOT NULL,
  "f20" TEXT NOT NULL,
  "f21" BYTEA NOT NULL,
  "f22" TEXT NOT NULL,
  "f23" BYTEA,
  "f24" BYTEA,
  "f25" TEXT
);
CREATE TABLE "pd_wafl__Holder" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f4" TEXT NOT NULL,
  "f5" TEXT NOT NULL,
  "f6" TEXT NOT NULL,
  "f7" TEXT NOT NULL,
  "f13" TEXT NOT NULL,
  "f14" TEXT,
  "f15" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Instance" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f4" TEXT NOT NULL,
  "f5" TEXT NOT NULL,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" BIGINT NOT NULL,
  "f11" TEXT NOT NULL,
  "f17" TEXT
);
CREATE TABLE "pd_wafl__Outbox" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f8" BYTEA NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" BYTEA NOT NULL,
  "f12" BYTEA NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BYTEA,
  "f17" BYTEA,
  "f18" BYTEA
);
CREATE TABLE "pd_wafl__Owed" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f8" BYTEA NOT NULL,
  "f9" TEXT NOT NULL,
  "f15" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__RowSeal" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f2" BYTEA NOT NULL,
  "f8" BYTEA NOT NULL,
  "f9" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Tenant" (
  "f1" BYTEA NOT NULL PRIMARY KEY,
  "f4" TEXT NOT NULL,
  "f5" TEXT NOT NULL,
  "f6" TEXT NOT NULL,
  "f7" TEXT NOT NULL,
  "f13" TEXT NOT NULL,
  "f15" TEXT NOT NULL
);
CREATE INDEX "pd_wafl__Access__access" ON "pd_wafl__Access" ("f2", "f15");
CREATE INDEX "pd_wafl__Access__by_actor" ON "pd_wafl__Access" ("f8", "f15");
CREATE INDEX "pd_wafl__Access__by_actor_tenant" ON "pd_wafl__Access" ("f16", "f15");
CREATE INDEX "pd_wafl__Access__by_time" ON "pd_wafl__Access" ("f15", "f1");
CREATE INDEX "pd_wafl__Archived__blob" ON "pd_wafl__Archived" ("f8", "f9");
CREATE INDEX "pd_wafl__Archived__state" ON "pd_wafl__Archived" ("f11");
CREATE INDEX "pd_wafl__Audit__by_actor" ON "pd_wafl__Audit" ("f8", "f15");
CREATE INDEX "pd_wafl__Audit__by_actor_tenant" ON "pd_wafl__Audit" ("f16", "f15");
CREATE INDEX "pd_wafl__Audit__by_counterpart" ON "pd_wafl__Audit" ("f18", "f15");
CREATE INDEX "pd_wafl__Audit__by_domain" ON "pd_wafl__Audit" ("f19", "f15");
CREATE INDEX "pd_wafl__Audit__by_tenant_domain" ON "pd_wafl__Audit" ("f2", "f19", "f15");
CREATE INDEX "pd_wafl__Audit__by_time" ON "pd_wafl__Audit" ("f15", "f1");
CREATE INDEX "pd_wafl__Audit__object" ON "pd_wafl__Audit" ("f11");
CREATE INDEX "pd_wafl__Audit__trail" ON "pd_wafl__Audit" ("f2", "f15");
ALTER TABLE "pd_wafl__Holder" ADD FOREIGN KEY ("f2") REFERENCES "pd_wafl__Tenant" ("f1");
CREATE INDEX "pd_wafl__Holder__page" ON "pd_wafl__Holder" ("f2", "f15", "f1");
CREATE UNIQUE INDEX "pd_wafl__Holder__slug" ON "pd_wafl__Holder" ("f4", "f2") WHERE "f14" IS NULL;
ALTER TABLE "pd_wafl__Instance" ADD FOREIGN KEY ("f2") REFERENCES "pd_wafl__Tenant" ("f1");
CREATE UNIQUE INDEX "pd_wafl__Instance__field_4" ON "pd_wafl__Instance" ("f4");
CREATE INDEX "pd_wafl__Outbox__queue" ON "pd_wafl__Outbox" ("f1");
CREATE INDEX "pd_wafl__Owed__owed" ON "pd_wafl__Owed" ("f15", "f1");
CREATE INDEX "pd_wafl__Tenant__page" ON "pd_wafl__Tenant" ("f15", "f1");
CREATE UNIQUE INDEX "pd_wafl__Tenant__field_4" ON "pd_wafl__Tenant" ("f4");
