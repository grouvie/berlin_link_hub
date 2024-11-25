-- Drop triggers and functions related to the tables
DROP TRIGGER IF EXISTS meetup_uris_trigger ON public.meetup_uris;
DROP FUNCTION IF EXISTS notify_meetup_uris_change();

-- Drop tables that depend on each other
DROP TABLE IF EXISTS public.uri_records CASCADE;

DROP TABLE IF EXISTS public.meetup_uris CASCADE;

-- Drop the 'users' table if it exists
DROP TABLE IF EXISTS public.users CASCADE;

-- Drop the 'user_roles' table if it exists
DROP TABLE IF EXISTS public.user_roles CASCADE;
