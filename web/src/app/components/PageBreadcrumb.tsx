import { Fragment } from 'react';
import { Link } from 'react-router-dom';
import { t as translate } from '../../shared/i18n';

/**
 * Editor-layer breadcrumb (IA_SHELL §2.3 / UI-SHL-10): the first segment is
 * the workspace and navigates back with a click.
 *
 * Segments starting with `crumb.` are registry keys and resolve through the
 * i18n dictionary; anything else renders verbatim (page titles keep their
 * English names).
 */
export function PageBreadcrumb({ trail }: { trail: string[] }) {
  return (
    <nav className="breadcrumb page-breadcrumb" aria-label="Breadcrumb">
      {trail.map((raw, index) => {
        const segment =
          raw === 'crumb.projects' || raw === 'crumb.editors' ? translate(raw) : raw;
        return (
          <Fragment key={`${raw}-${index}`}>
            {index > 0 && <span className="breadcrumb-sep">/</span>}
            {index === 0 ? (
              <Link to="/" className="breadcrumb-home">{segment}</Link>
            ) : (
              <span aria-current="page">{segment}</span>
            )}
          </Fragment>
        );
      })}
    </nav>
  );
}
