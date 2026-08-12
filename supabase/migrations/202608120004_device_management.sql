create table public.device_audit_events (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  device_kind text not null check (device_kind in ('connector', 'remote')),
  device_id uuid not null,
  remote_session_id uuid references public.remote_sessions(id) on delete set null,
  action text not null check (action in ('requested', 'accepted', 'denied', 'connected', 'closed', 'renamed', 'revoked')),
  mode text check (mode in ('view', 'control')),
  created_at timestamptz not null default now()
);

create index device_audit_events_owner_created_idx
  on public.device_audit_events (user_id, created_at desc);

alter table public.device_audit_events enable row level security;

create policy device_audit_events_owner_select
  on public.device_audit_events for select
  using ((select auth.uid()) = user_id);

create policy device_audit_events_owner_insert
  on public.device_audit_events for insert
  with check ((select auth.uid()) = user_id);
