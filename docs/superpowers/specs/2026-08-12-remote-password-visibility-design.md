# Remote PC password visibility

## Goal

Allow the person operating the local CommandBlock Remote PC console to opt in to seeing the password while entering it.

## Behaviour

Before the password prompt, Remote PC asks: `แสดงรหัสผ่านขณะพิมพ์หรือไม่? [y/N]:`.

- Enter or any answer other than `y`/`yes` keeps the existing masked password prompt.
- `y` or `yes` uses the normal visible console input prompt for that one login attempt.
- The password is never printed after entry, persisted, or sent anywhere except the existing authenticated sign-in request.

## Scope and verification

This changes only `src/remote.rs`; Desktop Connector keeps its masked-password behaviour. A unit test covers the explicit opt-in parsing so the safe, masked default cannot regress.
