-- Products we sell. The SKU (stock keeping unit) is the business key people type;
-- the integer id is what other tables point at.
-- This migration has already "shipped": never edit it. Change the schema with a new file.
CREATE TABLE products (
    id          INTEGER PRIMARY KEY,
    sku         TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    price_cents INTEGER NOT NULL CHECK (price_cents >= 0)
);
