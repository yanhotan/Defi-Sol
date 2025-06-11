"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"

interface RewardData {
  date: string
  amount: number
}

export function RewardsChart() {
  const [period, setPeriod] = useState("month")
  const [data, setData] = useState<RewardData[]>([])

  useEffect(() => {
    // Simulate fetching rewards data based on period
    const generateData = () => {
      const now = new Date()
      const result: RewardData[] = []

      let days = 30
      if (period === "week") days = 7
      if (period === "year") days = 365

      for (let i = days - 1; i >= 0; i--) {
        const date = new Date(now)
        date.setDate(date.getDate() - i)

        // Generate a random reward amount between 0.001 and 0.02 SOL
        const amount = 0.001 + Math.random() * 0.019

        result.push({
          date: date.toISOString().split("T")[0],
          amount: Number.parseFloat(amount.toFixed(6)),
        })
      }

      setData(result)
    }

    generateData()
  }, [period])

  // Calculate total rewards for the selected period
  const totalRewards = data.reduce((sum, item) => sum + item.amount, 0)

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Rewards History</CardTitle>
          <CardDescription>Your staking rewards over time</CardDescription>
        </div>
        <Select value={period} onValueChange={setPeriod}>
          <SelectTrigger className="w-[120px]">
            <SelectValue placeholder="Select period" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="week">Week</SelectItem>
            <SelectItem value="month">Month</SelectItem>
            <SelectItem value="year">Year</SelectItem>
          </SelectContent>
        </Select>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="chart" className="space-y-4">
          <TabsList>
            <TabsTrigger value="chart">Chart</TabsTrigger>
            <TabsTrigger value="summary">Summary</TabsTrigger>
          </TabsList>
          <TabsContent value="chart" className="space-y-4">
            <div className="h-[300px] w-full">
              <div className="flex h-full w-full items-center justify-center text-sm text-muted-foreground">
                Chart visualization would go here
              </div>
            </div>
          </TabsContent>
          <TabsContent value="summary">
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">Total Rewards</p>
                  <p className="text-2xl font-bold">{totalRewards.toFixed(6)} SOL</p>
                </div>
                <div className="space-y-1">
                  <p className="text-sm text-muted-foreground">Average Daily</p>
                  <p className="text-2xl font-bold">{(totalRewards / data.length).toFixed(6)} SOL</p>
                </div>
              </div>
              <div className="rounded-lg bg-muted p-4">
                <p className="text-sm">
                  Your rewards are automatically compounded, increasing your staking position over time.
                </p>
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  )
}
