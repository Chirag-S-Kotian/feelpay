import { render, screen } from "@testing-library/react";

import App from "./App";

describe("Admin App", () => {
  it("renders admin shell", async () => {
    render(<App />);
    expect((await screen.findAllByText(/FeelPay Admin/i)).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Tenants/i).length).toBeGreaterThan(0);
  });
});
