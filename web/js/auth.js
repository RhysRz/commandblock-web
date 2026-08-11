export function createAuthController(client, view) {
  async function signUp(email, password, displayName) {
    const { error } = await client.auth.signUp({
      email,
      password,
      options: { data: { full_name: displayName }, emailRedirectTo: location.href },
    });
    if (error) throw error;
    view.showNotice('ส่งอีเมลยืนยันบัญชีแล้ว กรุณาเปิดลิงก์ในอีเมล');
  }

  async function signIn(email, password) {
    const { data, error } = await client.auth.signInWithPassword({ email, password });
    if (error) throw error;
    if (data.session?.user) view.showAuthenticated(data.session.user, data.session);
  }

  async function sendPasswordReset(email) {
    const { error } = await client.auth.resetPasswordForEmail(email, { redirectTo: location.href });
    if (error) throw error;
    view.showNotice('ส่งลิงก์ตั้งรหัสผ่านใหม่แล้ว');
  }

  async function signOut() {
    const { error } = await client.auth.signOut();
    if (error) throw error;
  }

  client.auth.onAuthStateChange((_event, session) => session?.user ? view.showAuthenticated(session.user, session) : view.showUnauthenticated());
  return { signUp, signIn, sendPasswordReset, signOut };
}
