import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { HelpCircle, AlertTriangle } from "lucide-react"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"

export function StakingInfo() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Staking Information</CardTitle>
        <CardDescription>Learn more about staking SOL on our platform</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Accordion type="single" collapsible className="w-full">
          <AccordionItem value="item-1">
            <AccordionTrigger>How do the different pools work?</AccordionTrigger>
            <AccordionContent>
              <div className="space-y-4 text-sm">
                <div className="space-y-2">
                  <h4 className="font-medium">Basic Pool</h4>
                  <p className="text-muted-foreground">
                    The Basic Pool offers low-risk staking with instant unstaking capability. Your SOL is staked with
                    trusted validators, and you can withdraw at any time without waiting periods.
                  </p>
                </div>
                <div className="space-y-2">
                  <h4 className="font-medium">Lending Pool</h4>
                  <p className="text-muted-foreground">
                    The Lending Pool offers medium-risk staking where your SOL is used for both staking and lending to
                    borrowers. This generates additional yield but requires a 24-hour unstaking period.
                  </p>
                </div>
                <div className="space-y-2">
                  <h4 className="font-medium">Lock Pool</h4>
                  <p className="text-muted-foreground">
                    The Lock Pool offers high-risk staking with the highest yields. Your SOL is locked for a fixed term
                    (1 month, 3 months, 6 months, or 1 year) and cannot be withdrawn early. Longer lock periods provide
                    higher APY boosts, up to 25% APY for 1-year locks.
                  </p>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-2">
            <AccordionTrigger>What are liquid staking tokens (mSOL)?</AccordionTrigger>
            <AccordionContent>
              <p className="text-sm text-muted-foreground">
                When you stake SOL, you receive mSOL tokens that represent your staked position. These tokens can be
                transferred, traded, or used in DeFi applications while your original SOL remains staked and earning
                rewards. The value of mSOL increases over time relative to SOL as staking rewards accumulate.
              </p>
            </AccordionContent>
          </AccordionItem>
          <AccordionItem value="item-3">
            <AccordionTrigger>What are the risks?</AccordionTrigger>
            <AccordionContent>
              <div className="space-y-4 text-sm">
                <div className="flex gap-2">
                  <AlertTriangle className="h-5 w-5 text-yellow-500 shrink-0 mt-0.5" />
                  <div>
                    <h4 className="font-medium">Smart Contract Risk</h4>
                    <p className="text-muted-foreground">
                      All pools are subject to smart contract risks. Our contracts are audited, but no system is
                      completely risk-free.
                    </p>
                  </div>
                </div>
                <div className="flex gap-2">
                  <AlertTriangle className="h-5 w-5 text-yellow-500 shrink-0 mt-0.5" />
                  <div>
                    <h4 className="font-medium">Lending Pool Risks</h4>
                    <p className="text-muted-foreground">
                      The Lending Pool has additional risk from borrower defaults, though loans are over-collateralized.
                    </p>
                  </div>
                </div>
                <div className="flex gap-2">
                  <AlertTriangle className="h-5 w-5 text-red-500 shrink-0 mt-0.5" />
                  <div>
                    <h4 className="font-medium">Lock Pool Risks</h4>
                    <p className="text-muted-foreground">
                      The Lock Pool has the highest risk due to the inability to withdraw early and exposure to market
                      volatility during the lock period.
                    </p>
                  </div>
                </div>
              </div>
            </AccordionContent>
          </AccordionItem>
        </Accordion>

        <div className="rounded-lg bg-muted p-4">
          <div className="flex gap-2">
            <HelpCircle className="h-5 w-5 text-blue-500 shrink-0" />
            <div>
              <h4 className="font-medium">Need Help?</h4>
              <p className="text-sm text-muted-foreground">
                If you have any questions about staking pools, please refer to our documentation or contact our support
                team.
              </p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
