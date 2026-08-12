create table public.connector_devices (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  name text not null check (char_length(name) between 1 and 80),
  root_name text not null default 'โฟลเดอร์โปรเจกต์' check (char_length(root_name) between 1 and 160),
  last_seen_at timestamptz not null default now(),
  created_at timestamptz not null default now()
);

create table public.connector_commands (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade,
  device_id uuid not null references public.connector_devices(id) on delete cascade,
  action text not null check (action in ('files', 'read', 'changes', 'queue', 'pick_folder', 'exec', 'preview')),
  payload jsonb not null default '{}'::jsonb,
  status text not null default 'queued' check (status in ('queued', 'running', 'completed', 'rejected', 'failed')),
  result jsonb,
  error text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

alter table public.connector_devices enable row level security;
alter table public.connector_commands enable row level security;

create policy connector_devices_owner on public.connector_devices for all
  using ((select auth.uid()) = user_id)
  with check ((select auth.uid()) = user_id);

create policy connector_commands_owner on public.connector_commands for select
  using ((select auth.uid()) = user_id);
create policy connector_commands_owner_insert on public.connector_commands for insert
  with check (
    (select auth.uid()) = user_id
    and exists (
      select 1 from public.connector_devices d
      where d.id = device_id and d.user_id = (select auth.uid())
    )
  );
create policy connector_commands_owner_update on public.connector_commands for update
  using ((select auth.uid()) = user_id)
  with check ((select auth.uid()) = user_id);

create index connector_devices_owner_seen_idx
  on public.connector_devices (user_id, last_seen_at desc);
create index connector_commands_device_queue_idx
  on public.connector_commands (device_id, created_at)
  where status = 'queued';
create index connector_commands_owner_updated_idx
  on public.connector_commands (user_id, updated_at desc);

create function public.touch_connector_device() returns trigger
language plpgsql set search_path = public
as $$ begin new.last_seen_at = now(); return new; end; $$;
create trigger touch_connector_device_before_update
  before update on public.connector_devices
  for each row execute procedure public.touch_connector_device();

create function public.touch_connector_command() returns trigger
language plpgsql set search_path = public
as $$ begin new.updated_at = now(); return new; end; $$;
create trigger touch_connector_command_before_update
  before update on public.connector_commands
  for each row execute procedure public.touch_connector_command();
