"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { RefreshCw } from "lucide-react"

interface RewardEvent {
  id: string
  date: string
  amount: number
  validator: string
  status: "completed" | "pending"
}

export function RewardsHistory() {
  const [rewards, setRewards] = useState<RewardEvent[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // Simulate fetching rewards history
    setTimeout(() => {
      setRewards([
        {
          id: "1",
          date: "2023-04-15T10:30:00Z",
          amount: 0.05,
          validator: "Validator A",
          status: "completed",
        },
        {
          id: "2",
          date: "2023-04-14T08:15:00Z",
          amount: 0.04,
          validator: "Validator B",
          status: "completed",
        },
        {
          id: "3",
          date: "2023-04-13T14:45:00Z",
          amount: 0.06,
          validator: "Validator C",
          status: "completed",
        },
        {
          id: "4",
          date: "2023-04-12T09:20:00Z",
          amount: 0.05,
          validator: "Validator A",
          status: "completed",
        },
        {
          id: "5",
          date: "2023-04-11T16:10:00Z",
          amount: 0.04,
          validator: "Validator B",
          status: "completed",
        },
      ])
      setLoading(false)
    }, 1000)
  }, [])

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    })
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Rewards History</CardTitle>
            <CardDescription>Detailed history of your staking rewards</CardDescription>
          </div>
          <Button variant="outline" size="icon">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex h-[200px] items-center justify-center">
            <p className="text-sm text-muted-foreground">Loading rewards history...</p>
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Date</TableHead>
                <TableHead>Amount</TableHead>
                <TableHead>Validator</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rewards.map((reward) => (
                <TableRow key={reward.id}>
                  <TableCell>{formatDate(reward.date)}</TableCell>
                  <TableCell>{reward.amount} SOL</TableCell>
                  <TableCell>{reward.validator}</TableCell>
                  <TableCell>
                    <span
                      className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                        reward.status === "completed"
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400"
                      }`}
                    >
                      {reward.status === "completed" ? "Completed" : "Pending"}
                    </span>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
}
