import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Breadcrumb } from '../components/ui/breadcrumb';

const items = [
  { label: 'Home', href: '/' },
  { label: 'Discover', href: '/discover' },
  { label: 'Stellar Builders Summit' },
];

describe('Breadcrumb', () => {
  it('exposes a labelled navigation landmark', () => {
    render(<Breadcrumb items={items} />);

    expect(screen.getByRole('navigation', { name: 'breadcrumb' })).toBeInTheDocument();
  });

  it('renders ancestors as keyboard-navigable links', () => {
    render(<Breadcrumb items={items} />);

    const home = screen.getByRole('link', { name: 'Home' });
    const discover = screen.getByRole('link', { name: 'Discover' });

    expect(home).toHaveAttribute('href', '/');
    expect(discover).toHaveAttribute('href', '/discover');
  });

  it('marks the last item as the current page and does not link it', () => {
    render(<Breadcrumb items={items} />);

    const current = screen.getByText('Stellar Builders Summit');

    expect(current).toHaveAttribute('aria-current', 'page');
    expect(
      screen.queryByRole('link', { name: 'Stellar Builders Summit' }),
    ).not.toBeInTheDocument();
  });

  it('hides the separators from assistive technology', () => {
    const { container } = render(<Breadcrumb items={items} />);

    const separators = container.querySelectorAll('[aria-hidden="true"]');

    expect(separators).toHaveLength(items.length - 1);
  });

  it('renders nothing when given no items', () => {
    const { container } = render(<Breadcrumb items={[]} />);

    expect(container).toBeEmptyDOMElement();
  });
});
