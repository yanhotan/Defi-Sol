"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import Link from "next/link"
import { ArrowRight, CoinsIcon, DollarSign, Layers } from "lucide-react"

export function ProductsOverview() {
  const [productsData, setProductsData] = useState({
    msol: {
      active: 24.5,
      total: 30,
      apy: 7.2,
    },
    usdc: {
      active: 1250,
      total: 2000,
      apy: 5.5,
    },
    msolUsdc: {
      active: 15000,
      total: 20000,
      apy: 12.8,
    },
  })

  useEffect(() => {
    // Simulate fetching products data
  }, [])

  const msolProgressPercentage = (productsData.msol.active / productsData.msol.total) * 100
  const usdcProgressPercentage = (productsData.usdc.active / productsData.usdc.total) * 100
  const msolUsdcProgressPercentage = (productsData.msolUsdc.active / productsData.msolUsdc.total) * 100

  return (
    <Card>
      <CardHeader>
        <CardTitle>Products Overview</CardTitle>
        <CardDescription>Your current staking positions across products</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-solana-teal/10">
              <CoinsIcon className="h-4 w-4 text-solana-teal" />
            </div>
            <div>
              <h3 className="text-sm font-medium">mSOL Staking</h3>
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">APY:</span>
                <span className="text-xs font-medium text-solana-teal">{productsData.msol.apy}%</span>
              </div>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm">Active Stake</span>
              <span className="text-sm font-medium">
                {productsData.msol.active} / {productsData.msol.total} SOL
              </span>
            </div>
            <Progress value={msolProgressPercentage} className="h-2 bg-muted/50" indicatorClassName="bg-solana-teal" />
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-solana-blue/10">
              <DollarSign className="h-4 w-4 text-solana-blue" />
            </div>
            <div>
              <h3 className="text-sm font-medium">USDC Staking</h3>
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">APY:</span>
                <span className="text-xs font-medium text-solana-blue">{productsData.usdc.apy}%</span>
              </div>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm">Active Stake</span>
              <span className="text-sm font-medium">
                {productsData.usdc.active} / {productsData.usdc.total} USDC
              </span>
            </div>
            <Progress value={usdcProgressPercentage} className="h-2 bg-muted/50" indicatorClassName="bg-solana-blue" />
          </div>
        </div>

        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-solana-purple/10">
              <Layers className="h-4 w-4 text-solana-purple" />
            </div>
            <div>
              <h3 className="text-sm font-medium">mSOL-USDC Dual Staking</h3>
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">APY:</span>
                <span className="text-xs font-medium text-solana-purple">{productsData.msolUsdc.apy}%</span>
              </div>
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-sm">Active Stake</span>
              <span className="text-sm font-medium">
                ${(productsData.msolUsdc.active / 1000).toFixed(1)}k / $
                {(productsData.msolUsdc.total / 1000).toFixed(1)}k
              </span>
            </div>
            <Progress
              value={msolUsdcProgressPercentage}
              className="h-2 bg-muted/50"
              indicatorClassName="bg-solana-purple"
            />
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <Button asChild className="w-full bg-solana-gradient hover:opacity-90">
          <Link href="/products">
            View All Products
            <ArrowRight className="ml-2 h-4 w-4" />
          </Link>
        </Button>
      </CardFooter>
    </Card>
  )
}
