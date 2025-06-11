"use client"

import { useState } from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useToast } from "@/hooks/use-toast"

export function AccountSettings() {
  const [settings, setSettings] = useState({
    autoCompound: true,
    notifications: true,
    compoundFrequency: "daily",
  })
  const [isSaving, setIsSaving] = useState(false)
  const { toast } = useToast()

  const handleSaveSettings = () => {
    setIsSaving(true)

    // Simulate saving settings
    setTimeout(() => {
      toast({
        title: "Settings saved",
        description: "Your account settings have been updated",
      })
      setIsSaving(false)
    }, 1000)
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Account Settings</CardTitle>
        <CardDescription>Manage your staking preferences and notifications</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="auto-compound">Auto-Compound</Label>
              <p className="text-xs text-muted-foreground">Automatically reinvest your rewards</p>
            </div>
            <Switch
              id="auto-compound"
              checked={settings.autoCompound}
              onCheckedChange={(checked) => setSettings({ ...settings, autoCompound: checked })}
            />
          </div>
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="notifications">Notifications</Label>
              <p className="text-xs text-muted-foreground">Receive notifications for rewards and important events</p>
            </div>
            <Switch
              id="notifications"
              checked={settings.notifications}
              onCheckedChange={(checked) => setSettings({ ...settings, notifications: checked })}
            />
          </div>
        </div>
        <div className="space-y-2">
          <Label htmlFor="compound-frequency">Compound Frequency</Label>
          <Select
            value={settings.compoundFrequency}
            onValueChange={(value) => setSettings({ ...settings, compoundFrequency: value })}
            disabled={!settings.autoCompound}
          >
            <SelectTrigger id="compound-frequency">
              <SelectValue placeholder="Select frequency" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="daily">Daily</SelectItem>
              <SelectItem value="weekly">Weekly</SelectItem>
              <SelectItem value="monthly">Monthly</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </CardContent>
      <CardFooter>
        <Button onClick={handleSaveSettings} disabled={isSaving} className="w-full">
          {isSaving ? "Saving..." : "Save Settings"}
        </Button>
      </CardFooter>
    </Card>
  )
}
