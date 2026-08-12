create table public.usage_events (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  conversation_id uuid references public.conversations(id) on delete set null,
  model_id text not null check (char_length(model_id) between 1 and 120),
  prompt_tokens bigint not null default 0 check (prompt_tokens >= 0),
  completion_tokens bigint not null default 0 check (completion_tokens >= 0),
  total_tokens bigint not null default 0 check (total_tokens >= 0),
  exact boolean not null default false,
  cost_usd numeric(14, 8) not null default 0 check (cost_usd >= 0),
  created_at timestamptz not null default now()
);

create index usage_events_user_day_idx
  on public.usage_events (user_id, created_at desc);
create index usage_events_conversation_created_idx
  on public.usage_events (conversation_id, created_at desc);

alter table public.usage_events enable row level security;
create policy usage_events_owner_select on public.usage_events for select
  using ((select auth.uid()) = user_id);
create policy usage_events_owner_insert on public.usage_events for insert
  with check ((select auth.uid()) = user_id);

alter table public.remote_sessions
  add column approval_code_hash text,
  add column approval_code_input text,
  add column approval_expires_at timestamptz,
  add column approval_attempts integer not null default 0 check (approval_attempts between 0 and 5),
  add column host_verified_at timestamptz,
  add column closed_reason text;

create index remote_sessions_pending_approval_idx
  on public.remote_sessions (device_id, approval_expires_at desc)
  where status = 'requested';

alter table public.device_audit_events
  drop constraint if exists device_audit_events_action_check;
alter table public.device_audit_events
  add constraint device_audit_events_action_check check (action in (
    'requested', 'approval_requested', 'approval_failed', 'accepted', 'denied',
    'connected', 'closed', 'expired', 'renamed', 'revoked'
  ));
