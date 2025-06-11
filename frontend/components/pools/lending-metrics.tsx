"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { ArrowUpRight, ArrowDownRight } from "lucide-react"

export function LendingMetrics() {
  const [lendingData, setLendingData] = useState({
    totalSupplied: 2750000,
    totalBorrowed: 1870000,
    utilizationRate: 68,
    borrowAPR: 9.2,
    supplyAPR: 7.8,
    collateralRatio: 150,
    riskLevel: "Medium",
    weeklyChange: {
      utilization: 5,
      borrowAPR: -0.3,
      supplyAPR: 0.2,
    },
  })

  useEffect(() => {
    // Simulate fetching lending data
    // In a real app, this would be an API call
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Lending Pool Metrics</CardTitle>
        <CardDescription>Performance and utilization of the lending pool</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">Utilization Rate</h3>
            <div className="flex items-center gap-1">
              <span className="text-sm font-medium">{lendingData.utilizationRate}%</span>
              <span className={lendingData.weeklyChange.utilization >= 0 ? "text-green-500" : "text-red-500"}>
                {lendingData.weeklyChange.utilization >= 0 ? (
                  <ArrowUpRight className="h-3 w-3" />
                ) : (
                  <ArrowDownRight className="h-3 w-3" />
                )}
                {Math.abs(lendingData.weeklyChange.utilization)}%
              </span>
            </div>
          </div>
          <Progress value={lendingData.utilizationRate} className="h-2" />
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Total Supplied</p>
              <p className="text-sm font-medium">{(lendingData.totalSupplied / 1000000).toFixed(2)}M SOL</p>
            </div>
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Total Borrowed</p>
              <p className="text-sm font-medium">{(lendingData.totalBorrowed / 1000000).toFixed(2)}M SOL</p>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-6">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <p className="text-sm">Supply APR</p>
              <div className="flex items-center gap-1">
                <span className="text-sm font-medium">{lendingData.supplyAPR}%</span>
                <span className={lendingData.weeklyChange.supplyAPR >= 0 ? "text-green-500" : "text-red-500"}>
                  {lendingData.weeklyChange.supplyAPR >= 0 ? (
                    <ArrowUpRight className="h-3 w-3" />
                  ) : (
                    <ArrowDownRight className="h-3 w-3" />
                  )}
                  {Math.abs(lendingData.weeklyChange.supplyAPR)}%
                </span>
              </div>
            </div>
            <Progress value={lendingData.supplyAPR * 10} className="h-1.5" />
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <p className="text-sm">Borrow APR</p>
              <div className="flex items-center gap-1">
                <span className="text-sm font-medium">{lendingData.borrowAPR}%</span>
                <span className={lendingData.weeklyChange.borrowAPR >= 0 ? "text-red-500" : "text-green-500"}>
                  {lendingData.weeklyChange.borrowAPR >= 0 ? (
                    <ArrowUpRight className="h-3 w-3" />
                  ) : (
                    <ArrowDownRight className="h-3 w-3" />
                  )}
                  {Math.abs(lendingData.weeklyChange.borrowAPR)}%
                </span>
              </div>
            </div>
            <Progress value={lendingData.borrowAPR * 10} className="h-1.5" />
          </div>
        </div>

        <div className="rounded-lg bg-muted p-4">
          <h3 className="text-sm font-medium mb-2">Lending Pool Information</h3>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Collateral Ratio</p>
              <p>{lendingData.collateralRatio}%</p>
            </div>
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Risk Level</p>
              <p className="text-yellow-500">{lendingData.riskLevel}</p>
            </div>
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Interest Model</p>
              <p>Dynamic</p>
            </div>
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Liquidation Threshold</p>
              <p>120%</p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
