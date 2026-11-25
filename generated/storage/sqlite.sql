CREATE TABLE "pd_wafl__Access" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL,
  "f8" BLOB NOT NULL,
  "f9" BLOB NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" TEXT NOT NULL,
  "f12" TEXT NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BLOB NOT NULL,
  "f17" BLOB NOT NULL,
  "f19" BIGINT NOT NULL,
  "f20" TEXT NOT NULL,
  "f21" BIGINT NOT NULL
);
CREATE TABLE "pd_wafl__Archived" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f7" TEXT NOT NULL,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" TEXT NOT NULL,
  "f12" BIGINT NOT NULL,
  "f13" TEXT NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BIGINT NOT NULL,
  "f17" BLOB NOT NULL,
  "f18" BLOB NOT NULL
);
CREATE TABLE "pd_wafl__Audit" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL,
  "f8" BLOB NOT NULL,
  "f9" BLOB NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" BLOB NOT NULL,
  "f12" BLOB NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BLOB NOT NULL,
  "f17" BLOB NOT NULL,
  "f18" BLOB,
  "f19" BIGINT NOT NULL,
  "f20" TEXT NOT NULL,
  "f21" BLOB NOT NULL,
  "f22" TEXT NOT NULL,
  "f23" BLOB,
  "f24" BLOB,
  "f25" TEXT
);
CREATE TABLE "pd_wafl__Holder" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL REFERENCES "pd_wafl__Tenant" ("f1"),
  "f4" TEXT NOT NULL,
  "f5" TEXT NOT NULL,
  "f6" TEXT NOT NULL,
  "f7" TEXT NOT NULL,
  "f13" TEXT NOT NULL,
  "f14" TEXT,
  "f15" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Instance" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL REFERENCES "pd_wafl__Tenant" ("f1"),
  "f4" TEXT NOT NULL,
  "f5" TEXT NOT NULL,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" BIGINT NOT NULL,
  "f11" TEXT NOT NULL,
  "f17" TEXT,
  "f18" BLOB NOT NULL,
  "f19" TEXT NOT NULL,
  "f20" BOOLEAN NOT NULL,
  "f21" TEXT,
  "f22" BOOLEAN NOT NULL
);
CREATE TABLE "pd_wafl__Outbox" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL,
  "f8" BLOB NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" BLOB NOT NULL,
  "f12" BLOB NOT NULL,
  "f15" TEXT NOT NULL,
  "f16" BLOB,
  "f17" BLOB,
  "f18" BLOB
);
CREATE TABLE "pd_wafl__Owed" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f8" BLOB NOT NULL,
  "f9" TEXT NOT NULL,
  "f15" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Reservation" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" BLOB NOT NULL,
  "f11" BLOB NOT NULL,
  "f12" TEXT NOT NULL,
  "f16" TEXT NOT NULL,
  "f17" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__RowSeal" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f2" BLOB NOT NULL,
  "f8" BLOB NOT NULL,
  "f9" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Runner" (
  "f1" BLOB NOT NULL PRIMARY KEY,
  "f4" TEXT NOT NULL,
  "f8" TEXT NOT NULL,
  "f9" TEXT NOT NULL,
  "f10" TEXT NOT NULL,
  "f11" TEXT NOT NULL,
  "f12" TEXT NOT NULL,
  "f16" BOOLEAN NOT NULL,
  "f17" TEXT,
  "f18" TEXT,
  "f19" TEXT NOT NULL
);
CREATE TABLE "pd_wafl__Tenant" (
  "f1" BLOB NOT NULL PRIMARY KEY,
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
CREATE INDEX "pd_wafl__Holder__page" ON "pd_wafl__Holder" ("f2", "f15", "f1");
CREATE UNIQUE INDEX "pd_wafl__Holder__slug" ON "pd_wafl__Holder" ("f4", "f2") WHERE "f14" IS NULL;
CREATE UNIQUE INDEX "pd_wafl__Instance__field_4" ON "pd_wafl__Instance" ("f4");
CREATE INDEX "pd_wafl__Outbox__queue" ON "pd_wafl__Outbox" ("f1");
CREATE INDEX "pd_wafl__Owed__owed" ON "pd_wafl__Owed" ("f15", "f1");
CREATE UNIQUE INDEX "pd_wafl__Reservation__field_8" ON "pd_wafl__Reservation" ("f8");
CREATE UNIQUE INDEX "pd_wafl__Runner__field_4" ON "pd_wafl__Runner" ("f4");
CREATE INDEX "pd_wafl__Tenant__page" ON "pd_wafl__Tenant" ("f15", "f1");
CREATE UNIQUE INDEX "pd_wafl__Tenant__field_4" ON "pd_wafl__Tenant" ("f4");
