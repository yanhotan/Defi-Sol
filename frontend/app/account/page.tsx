import { AccountDetails } from "@/components/account/account-details"
import { AccountSettings } from "@/components/account/account-settings"
import { WalletConnect } from "@/components/wallet-connect"

export default function AccountPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Account</h1>
        <p className="text-muted-foreground">Manage your account and wallet settings</p>
      </div>
      <WalletConnect />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <AccountDetails />
        <AccountSettings />
      </div>
    </div>
  )
}
