create table public.remote_devices (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  name text not null check (char_length(name) between 1 and 80),
  last_seen_at timestamptz not null default now(),
  created_at timestamptz not null default now()
);

create table public.remote_sessions (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  device_id uuid not null references public.remote_devices(id) on delete cascade,
  mode text not null check (mode in ('view', 'control')),
  status text not null default 'requested' check (status in ('requested', 'accepted', 'connected', 'denied', 'closed', 'expired')),
  offer jsonb,
  answer jsonb,
  expires_at timestamptz not null default (now() + interval '10 minutes'),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index remote_devices_owner_seen_idx on public.remote_devices (user_id, last_seen_at desc);
create index remote_sessions_active_device_idx on public.remote_sessions (device_id, expires_at desc) where status in ('requested', 'accepted', 'connected');

alter table public.remote_devices enable row level security;
alter table public.remote_sessions enable row level security;

create policy remote_devices_owner on public.remote_devices for all using ((select auth.uid()) = user_id) with check ((select auth.uid()) = user_id);
create policy remote_sessions_owner on public.remote_sessions for all using ((select auth.uid()) = user_id) with check (
  (select auth.uid()) = user_id
  and exists (select 1 from public.remote_devices d where d.id = device_id and d.user_id = (select auth.uid()))
);

create function public.touch_remote_session() returns trigger language plpgsql security invoker as $$
begin new.updated_at = now(); return new; end;
$$;
create trigger remote_sessions_updated before update on public.remote_sessions for each row execute function public.touch_remote_session();
