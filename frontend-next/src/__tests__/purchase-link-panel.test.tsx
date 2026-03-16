import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { handlers, mockPurchaseLinks } from '@/__mocks__/handlers';
import { PurchaseLinkPanel } from '@/components/purchase-link-panel';

const server = setupServer(...handlers);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe('PurchaseLinkPanel', () => {
  it('renders collapsed by default with Buy button', () => {
    render(
      <PurchaseLinkPanel title="Strings of Life" artist="Derrick May" />,
      { wrapper: createWrapper() },
    );
    expect(screen.getByText('Buy')).toBeInTheDocument();
    // Links should not be visible when collapsed
    expect(screen.queryByText('Beatport')).not.toBeInTheDocument();
  });

  it('expands on button click', async () => {
    render(
      <PurchaseLinkPanel title="Strings of Life" artist="Derrick May" />,
      { wrapper: createWrapper() },
    );
    fireEvent.click(screen.getByText('Buy'));
    // After expanding, should show loading or links
    await waitFor(() => {
      expect(screen.getByText('Beatport')).toBeInTheDocument();
    });
  });

  it('shows loading state while fetching', () => {
    // Delay the response to capture loading state
    server.use(
      http.get('/api/purchase-links', async () => {
        await new Promise((r) => setTimeout(r, 100));
        return HttpResponse.json({ links: mockPurchaseLinks });
      }),
    );
    render(
      <PurchaseLinkPanel title="Strings of Life" artist="Derrick May" />,
      { wrapper: createWrapper() },
    );
    fireEvent.click(screen.getByText('Buy'));
    // The Loader2 spinner should be in the DOM (it's an svg with animate-spin class)
    const spinner = document.querySelector('.animate-spin');
    expect(spinner).toBeInTheDocument();
  });

  it('renders all store links after fetch', async () => {
    render(
      <PurchaseLinkPanel title="Strings of Life" artist="Derrick May" />,
      { wrapper: createWrapper() },
    );
    fireEvent.click(screen.getByText('Buy'));

    await waitFor(() => {
      expect(screen.getByText('Beatport')).toBeInTheDocument();
      expect(screen.getByText('Bandcamp')).toBeInTheDocument();
      expect(screen.getByText('Juno Download')).toBeInTheDocument();
      expect(screen.getByText('Traxsource')).toBeInTheDocument();
    });
  });

  it('collapses when button is clicked again', async () => {
    render(
      <PurchaseLinkPanel title="Strings of Life" artist="Derrick May" />,
      { wrapper: createWrapper() },
    );
    fireEvent.click(screen.getByText('Buy'));
    await waitFor(() => {
      expect(screen.getByText('Beatport')).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText('Buy'));
    expect(screen.queryByText('Beatport')).not.toBeInTheDocument();
  });

  it('returns null when title and artist are empty', () => {
    const { container } = render(
      <PurchaseLinkPanel title="" artist="" />,
      { wrapper: createWrapper() },
    );
    expect(container.firstChild).toBeNull();
  });

  it('shows no purchase links message when empty', async () => {
    server.use(
      http.get('/api/purchase-links', () =>
        HttpResponse.json({ links: [] }),
      ),
    );
    render(
      <PurchaseLinkPanel title="Unknown Track" artist="Nobody" />,
      { wrapper: createWrapper() },
    );
    fireEvent.click(screen.getByText('Buy'));

    await waitFor(() => {
      expect(screen.getByText('No purchase links')).toBeInTheDocument();
    });
  });
});
