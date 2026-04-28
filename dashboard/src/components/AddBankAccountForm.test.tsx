import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";

import AddBankAccountForm from "./AddBankAccountForm";

vi.mock("../hooks/useApi", () => ({
  apiRequest: vi.fn().mockResolvedValue({}),
}));

describe("AddBankAccountForm", () => {
  it("shows validation errors for invalid form submit", async () => {
    render(<AddBankAccountForm onClose={() => {}} />);

    fireEvent.click(screen.getByRole("button", { name: /add account/i }));

    expect(await screen.findByText(/account holder name is required/i)).toBeInTheDocument();
    expect(screen.getByText(/account number is required/i)).toBeInTheDocument();
    expect(screen.getByText(/ifsc code is required/i)).toBeInTheDocument();
  });

  it("accepts valid IFSC/account format", async () => {
    render(<AddBankAccountForm onClose={() => {}} />);

    fireEvent.change(screen.getByPlaceholderText(/john doe/i), {
      target: { value: "John Doe" },
    });
    fireEvent.change(screen.getByPlaceholderText(/1234567890123456/i), {
      target: { value: "123456789012" },
    });
    fireEvent.change(screen.getByPlaceholderText(/hdfc0001234/i), {
      target: { value: "hdfc0001234" },
    });

    fireEvent.click(screen.getByRole("button", { name: /add account/i }));

    expect(screen.queryByText(/invalid ifsc code format/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/invalid account number format/i)).not.toBeInTheDocument();
  });
});
