import { render, screen } from "@testing-library/react";

import App from "./App";

describe("Admin App", () => {
  beforeEach(() => {
    const claims = btoa(JSON.stringify({ role: "Super Admin", permissions: [] }));
    const store = new Map<string, string>([["feelpay_token", `test.${claims}.token`]]);
    Object.defineProperty(globalThis, "localStorage", {
      value: {
        getItem: (key: string) => store.get(key) || null,
        setItem: (key: string, value: string) => store.set(key, value),
        removeItem: (key: string) => store.delete(key),
        clear: () => store.clear(),
      },
      writable: true,
    });
    window.history.pushState({}, "", "/");
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("renders admin shell", async () => {
    render(<App />);
    expect((await screen.findAllByText(/FeelPay/i)).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Business Accounts/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Ledger/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Refunds/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Admin Users & Roles/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/System Health/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Notifications/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Platform Settings/i).length).toBeGreaterThan(0);
  });
});
