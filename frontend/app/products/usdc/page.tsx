import { ProductHeader } from "@/components/products/product-header"
import { StakingForm } from "@/components/products/staking-form"
import { ProductInfo } from "@/components/products/product-info"
import { WalletConnect } from "@/components/wallet-connect"

export default function UsdcProductPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <ProductHeader title="USDC Staking" description="Stake your USDC to earn stable yields" productType="usdc" />
      <WalletConnect />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <StakingForm productType="usdc" />
        <ProductInfo productType="usdc" />
      </div>
    </div>
  )
}
