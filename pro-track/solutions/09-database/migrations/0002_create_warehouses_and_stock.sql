-- Warehouses, and how many of each product each one holds.
CREATE TABLE warehouses (
    id   INTEGER PRIMARY KEY,
    name TEXT    NOT NULL UNIQUE
);

-- One row per (warehouse, product) pair. The composite primary key makes the pair unique,
-- which is what INSERT ... ON CONFLICT (warehouse_id, product_id) relies on.
CREATE TABLE stock (
    warehouse_id INTEGER NOT NULL REFERENCES warehouses (id),
    product_id   INTEGER NOT NULL REFERENCES products (id),
    quantity     INTEGER NOT NULL CHECK (quantity >= 0),
    PRIMARY KEY (warehouse_id, product_id)
);
