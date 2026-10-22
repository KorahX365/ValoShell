// Wallet balances for the top bar.
import { createSignal } from "solid-js";
import type { WalletView } from "~/bindings/WalletView";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";

const [wallet, setWallet] = createSignal<WalletView | null>(null);

onAppEvent("wallet", setWallet);
onAppEvent("connection", (status) => status.state !== "connected" && setWallet(null));
void call("get_wallet", {}).then((current) => current && setWallet(current), () => undefined);

export { wallet };
