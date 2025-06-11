import { ProductsHeader } from "@/components/products/products-header"
import { ProductsList } from "@/components/products/products-list"
import { ProductsStats } from "@/components/products/products-stats"
import { WalletConnect } from "@/components/wallet-connect"

export default function ProductsPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <ProductsHeader />
      <WalletConnect />
      <ProductsStats />
      <ProductsList />
    </div>
  )
}
