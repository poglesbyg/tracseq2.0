CREATE TYPE sample_status AS ENUM ('pending', 'validated', 'in_storage', 'in_sequencing', 'completed');

CREATE TABLE samples (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    barcode VARCHAR(50) NOT NULL UNIQUE,
    location VARCHAR(255) NOT NULL,
    status sample_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_samples_barcode ON samples(barcode);
CREATE INDEX idx_samples_status ON samples(status);
CREATE INDEX idx_samples_location ON samples(location); 
