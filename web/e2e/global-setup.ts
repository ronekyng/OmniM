import { mkdirSync, rmSync } from 'node:fs';

/** Isolated project root so e2e runs never touch the developer's workspace. */
export default function globalSetup(): void {
  rmSync('/tmp/opencode/omnim-e2e', { recursive: true, force: true });
  mkdirSync('/tmp/opencode/omnim-e2e', { recursive: true });
}
