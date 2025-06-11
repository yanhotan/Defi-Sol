"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ArrowDownToLine, ArrowUpFromLine, RefreshCw } from "lucide-react"

type ActivityType = "stake" | "unstake" | "reward"

interface Activity {
  id: string
  type: ActivityType
  amount: number
  timestamp: string
  status: "completed" | "pending"
}

export function RecentActivity() {
  const [activities, setActivities] = useState<Activity[]>([])

  useEffect(() => {
    // Simulate fetching activity data
    setActivities([
      {
        id: "1",
        type: "stake",
        amount: 5,
        timestamp: "2023-04-15T10:30:00Z",
        status: "completed",
      },
      {
        id: "2",
        type: "reward",
        amount: 0.12,
        timestamp: "2023-04-14T08:15:00Z",
        status: "completed",
      },
      {
        id: "3",
        type: "unstake",
        amount: 2.5,
        timestamp: "2023-04-10T14:45:00Z",
        status: "completed",
      },
      {
        id: "4",
        type: "stake",
        amount: 10,
        timestamp: "2023-04-05T09:20:00Z",
        status: "completed",
      },
    ])
  }, [])

  const getActivityIcon = (type: ActivityType) => {
    switch (type) {
      case "stake":
        return <ArrowDownToLine className="h-4 w-4 text-green-500" />
      case "unstake":
        return <ArrowUpFromLine className="h-4 w-4 text-red-500" />
      case "reward":
        return <RefreshCw className="h-4 w-4 text-yellow-500" />
    }
  }

  const getActivityText = (type: ActivityType) => {
    switch (type) {
      case "stake":
        return "Staked"
      case "unstake":
        return "Unstaked"
      case "reward":
        return "Reward"
    }
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    })
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Recent Activity</CardTitle>
        <CardDescription>Your recent staking transactions and rewards</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {activities.length === 0 ? (
            <p className="text-center text-sm text-muted-foreground py-4">No recent activity</p>
          ) : (
            activities.map((activity) => (
              <div
                key={activity.id}
                className="flex items-center justify-between border-b border-border pb-3 last:border-0 last:pb-0"
              >
                <div className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-muted">
                    {getActivityIcon(activity.type)}
                  </div>
                  <div>
                    <p className="text-sm font-medium">
                      {getActivityText(activity.type)} {activity.amount} SOL
                    </p>
                    <p className="text-xs text-muted-foreground">{formatDate(activity.timestamp)}</p>
                  </div>
                </div>
                <div>
                  <span
                    className={`text-xs font-medium ${
                      activity.status === "completed" ? "text-green-500" : "text-yellow-500"
                    }`}
                  >
                    {activity.status === "completed" ? "Completed" : "Pending"}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  )
}
