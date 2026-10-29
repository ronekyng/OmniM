import { Component, type ErrorInfo, type ReactNode } from 'react';

type Props = {
  children: ReactNode;
  /** Short label shown in the fallback ("Properties", "Workspace", ...). */
  scope?: string;
  /** Changing this value clears a caught error (route/panel reset). */
  resetKey?: unknown;
  /** Optional retry hook; when omitted only local state resets. */
  onRetry?: () => void;
};

type State = { error: Error | null; info: string | null };

/** Route- and panel-scoped error boundary (UI-FBK-06).
 *
 *  A panel exception renders in place with the shell chrome intact; a route
 *  exception offers a reload. Nothing here ever blanks the whole application.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, info: null };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    this.setState({ info: info.componentStack ?? null });
    console.error(`[${this.props.scope ?? 'ui'}] uncaught render error`, error, info);
  }

  componentDidUpdate(prev: Props) {
    if (prev.resetKey !== this.props.resetKey && this.state.error) {
      this.setState({ error: null, info: null });
    }
  }

  private retry = () => {
    this.props.onRetry?.();
    this.setState({ error: null, info: null });
  };

  render() {
    const { error, info } = this.state;
    if (!error) return this.props.children;
    const scope = this.props.scope ?? 'This panel';
    return (
      <div className="error-boundary" role="alert">
        <h3>{scope} crashed</h3>
        <p>The rest of the workspace is still usable. Retry to remount this area.</p>
        <p className="error-boundary-code">
          {error.name}: {error.message}
        </p>
        <div className="confirm-actions">
          <button type="button" onClick={this.retry}>
            Retry
          </button>
          <button type="button" onClick={() => window.location.reload()}>
            Reload app
          </button>
        </div>
        {info && (
          <details>
            <summary>Details</summary>
            <pre>{info}</pre>
          </details>
        )}
      </div>
    );
  }
}
