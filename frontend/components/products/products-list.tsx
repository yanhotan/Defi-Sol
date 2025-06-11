"use client"

import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { CoinsIcon, DollarSign, Layers, ArrowRight } from "lucide-react"
import Link from "next/link"

export function ProductsList() {
  const products = [
    {
      id: "msol",
      name: "mSOL Staking",
      description: "Stake your SOL to receive mSOL and earn rewards with different risk levels",
      icon: CoinsIcon,
      iconColor: "text-solana-teal",
      iconBg: "bg-solana-teal/10",
      apy: {
        low: "5.2%",
        medium: "7.8%",
        high: "12.5%",
      },
      riskLevels: ["Low", "Medium", "High"],
      href: "/products/msol",
    },
    {
      id: "usdc",
      name: "USDC Staking",
      description: "Stake your USDC to earn stable yields",
      icon: DollarSign,
      iconColor: "text-solana-blue",
      iconBg: "bg-solana-blue/10",
      apy: {
        standard: "5.5%",
      },
      riskLevels: ["Low"],
      href: "/products/usdc",
    },
    {
      id: "msol-usdc",
      name: "mSOL-USDC Dual Staking",
      description: "Stake both mSOL and USDC to earn enhanced yields with optional LP rewards",
      icon: Layers,
      iconColor: "text-solana-purple",
      iconBg: "bg-solana-purple/10",
      apy: {
        low: "8.5%",
        medium: "12.8%",
        high: "18.5%",
      },
      riskLevels: ["Low", "Medium", "High"],
      href: "/products/msol-usdc",
    },
  ]

  return (
    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {products.map((product) => (
        <Card key={product.id} className="flex flex-col">
          <CardHeader>
            <div className="flex items-center gap-3 mb-3">
              <div className={`flex h-10 w-10 items-center justify-center rounded-full ${product.iconBg}`}>
                <product.icon className={`h-5 w-5 ${product.iconColor}`} />
              </div>
              <CardTitle>{product.name}</CardTitle>
            </div>
            <CardDescription>{product.description}</CardDescription>
          </CardHeader>
          <CardContent className="flex-1">
            <div className="space-y-4">
              <div>
                <h4 className="text-sm font-medium mb-2">APY Range</h4>
                <div className="flex flex-wrap gap-2">
                  {product.id === "usdc" ? (
                    <Badge variant="outline" className="bg-solana-blue/10 text-solana-blue">
                      {product.apy.standard}
                    </Badge>
                  ) : (
                    <>
                      <Badge variant="outline" className="bg-solana-teal/10 text-solana-teal">
                        {product.apy.low} (Low Risk)
                      </Badge>
                      <Badge variant="outline" className="bg-solana-blue/10 text-solana-blue">
                        {product.apy.medium} (Medium Risk)
                      </Badge>
                      <Badge variant="outline" className="bg-solana-purple/10 text-solana-purple">
                        {product.apy.high} (High Risk)
                      </Badge>
                    </>
                  )}
                </div>
              </div>

              {product.id === "msol-usdc" && (
                <div>
                  <h4 className="text-sm font-medium mb-2">Features</h4>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="outline" className="bg-solana-gradient/10">
                      LP Rewards
                    </Badge>
                    <Badge variant="outline" className="bg-solana-gradient/10">
                      Swap Integration
                    </Badge>
                    <Badge variant="outline" className="bg-solana-gradient/10">
                      Dual Yield
                    </Badge>
                  </div>
                </div>
              )}
            </div>
          </CardContent>
          <CardFooter>
            <Button asChild className="w-full bg-solana-gradient hover:opacity-90">
              <Link href={product.href}>
                View Product
                <ArrowRight className="ml-2 h-4 w-4" />
              </Link>
            </Button>
          </CardFooter>
        </Card>
      ))}
    </div>
  )
}
