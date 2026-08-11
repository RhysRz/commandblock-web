export function getSupabaseConfig() {
  const url = document.querySelector('meta[name="supabase-url"]')?.content?.trim();
  const anonKey = document.querySelector('meta[name="supabase-anon-key"]')?.content?.trim();
  if (!url || !anonKey || url.includes('YOUR_') || anonKey.includes('YOUR_')) throw new Error('ตั้งค่า Supabase URL และ anon key ก่อนเริ่มใช้งาน');
  return { url, anonKey };
}
