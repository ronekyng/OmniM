import { Fragment } from 'react';
import { Link } from 'react-router-dom';

/**
 * Editor-layer breadcrumb (IA_SHELL §2.3 / UI-SHL-10): the first segment is
 * the workspace and navigates back with a click.
 */
export function PageBreadcrumb({ trail }: { trail: string[] }) {
  return (
    <nav className="breadcrumb page-breadcrumb" aria-label="Breadcrumb">
      {trail.map((segment, index) => (
        <Fragment key={`${segment}-${index}`}>
          {index > 0 && <span className="breadcrumb-sep">/</span>}
          {index === 0 ? (
            <Link to="/" className="breadcrumb-home">{segment}</Link>
          ) : (
            <span aria-current="page">{segment}</span>
          )}
        </Fragment>
      ))}
    </nav>
  );
}
