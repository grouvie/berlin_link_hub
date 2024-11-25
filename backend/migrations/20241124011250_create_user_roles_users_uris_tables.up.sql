-- Create the 'user_roles' table
CREATE TABLE public.user_roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT
);

-- Prefill 'user_roles' table
INSERT INTO public.user_roles (name, description)
VALUES 
    ('admin', 'Administrator with full permissions'),
    ('moderator', 'User with permission to moderate content'),
    ('guest', 'User with limited access to the platform');

-- Create the 'users' table
CREATE TABLE public.users (
    id SERIAL PRIMARY KEY,
    role_id INT REFERENCES user_roles(id) ON DELETE SET NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    username VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Dev only: Prefill an admin user into the 'users' table
-- TODO: REMOVE before production deployment
INSERT INTO public.users (role_id, email, password, username)
VALUES (
    (SELECT id FROM public.user_roles WHERE name = 'admin'),
    'admin@example.com',
    '$argon2d$v=19$m=16,t=2,p=1$cTdzWUJjMGFmRDB4RVRuNQ$nllqyj2CadDpydSSPozmbw',    -- "password" (argon2 hashed)
    'admin'
);

-- Create the 'meetup_uris' table
CREATE TABLE public.meetup_uris (
    id SERIAL PRIMARY KEY,
    meetup_date DATE NOT NULL,
    uri TEXT NOT NULL,
    created_by INT REFERENCES public.users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    handled BOOLEAN DEFAULT FALSE
);

-- Create the 'uri_records' table
CREATE TABLE public.uri_records (
    id SERIAL PRIMARY KEY,
    meetup_id INT REFERENCES meetup_uris(id) ON DELETE SET NULL,
    url TEXT NOT NULL,
    url_scheme VARCHAR(10) NOT NULL,
    url_host VARCHAR(255) NOT NULL,
    url_path TEXT,
    status BOOLEAN NOT NULL DEFAULT FALSE,
    title VARCHAR(255),
    auto_description TEXT,
    manual_description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_by INT REFERENCES public.users(id) ON DELETE CASCADE,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Trigger function to notify changes in meetup_uris
CREATE OR REPLACE FUNCTION notify_meetup_uris_change()
RETURNS TRIGGER AS $$
DECLARE
    highest_id INT;
    payload JSON;
BEGIN

    SELECT MAX(id) INTO highest_id
    FROM public.meetup_uris;
    
    payload := json_build_object(
        'id', highest_id,
        'timestamp', CURRENT_TIMESTAMP
    );

    PERFORM pg_notify('meetup_uris_change', payload::TEXT);

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for meetup_uris to notify on changes (FOR EACH STATEMENT)
CREATE TRIGGER meetup_uris_trigger
AFTER INSERT ON public.meetup_uris
FOR EACH STATEMENT EXECUTE FUNCTION notify_meetup_uris_change();
