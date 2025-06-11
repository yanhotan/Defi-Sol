import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { HelpCircle, AlertTriangle } from "lucide-react"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import type { ProductType } from "@/components/products/risk-pool-selector"

interface ProductInfoProps {
  productType: ProductType
}

export function ProductInfo({ productType }: ProductInfoProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>
          {productType === "msol"
            ? "mSOL Staking Information"
            : productType === "usdc"
              ? "USDC Staking Information"
              : "mSOL-USDC Dual Staking Information"}
        </CardTitle>
        <CardDescription>
          {productType === "msol"
            ? "Learn more about staking SOL for mSOL"
            : productType === "usdc"
              ? "Learn more about staking USDC"
              : "Learn more about dual staking mSOL and USDC"}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Accordion type="single" collapsible className="w-full">
          <AccordionItem value="item-1">
            <AccordionTrigger>
              How does{" "}
              {productType === "msol" ? "mSOL staking" : productType === "usdc" ? "USDC staking" : "dual staking"} work?
            </AccordionTrigger>
            <AccordionContent>
              <div className="space-y-4 text-sm">
                {productType === "msol" && (
                  <>
                    <p className="text-muted-foreground">
                      When you stake SOL, you receive mSOL tokens that represent your staked position. These tokens can
                      be transferred, traded, or used in DeFi applications while your original SOL remains staked and
                      earning rewards.
                    </p>
                    <div className="space-y-2">
                      <h4 className="font-medium">Risk Levels</h4>
                      <p className="text-muted-foreground">
                        <strong>Low Risk:</strong> Your SOL is staked with trusted validators with instant unstaking
                        capability.
                      </p>
                      <p className="text-muted-foreground">
                        <strong>Medium Risk:</strong> Your SOL is used for both staking and lending, with a 7-day
                        unstaking period.
                      </p>
                      <p className="text-muted-foreground">
                        <strong>High Risk:</strong> Your SOL is locked for a fixed term (1 month to 1 year) for higher
                        yields.
                      </p>
                    </div>
                  </>
                )}

                {productType === "usdc" && (
                  <p className="text-muted-foreground">
                    When you stake USDC, you receive stUSDC tokens that represent your staked position. Your USDC is
                    deployed to secure lending protocols to generate yield. This product offers stable returns with low
                    risk.
                  </p>
                )}

                {productType === "msol-usdc" && (
                  <>
                    <p className="text-muted-foreground">
                      The mSOL-USDC dual staking product allows you to stake both mSOL and USDC together. You can either
                      provide both assets separately or swap your SOL to get both assets in one transaction.
                    </p>
                    <div className="space-y-2">
                      <h4 className="font-medium">Risk Levels</h4>
                      <p className="text-muted-foreground">
                        <strong>Low Risk:</strong> Your assets are staked with instant unstaking capability.
                      </p>
                      <p className="text-muted-foreground">
                        <strong>Medium Risk:</strong> Your assets can be deployed to liquidity pools (LP) for additional
                        yield, with a 7-day unstaking period.
                      </p>
                      <p className="text-muted-foreground">
                        <strong>High Risk:</strong> Your assets are locked for a fixed term and deployed to LPs for
                        maximum yield.
                      </p>
                    </div>
                  </>
                )}
              </div>
            </AccordionContent>
          </AccordionItem>

          {productType === "msol-usdc" && (
            <AccordionItem value="item-2">
              <AccordionTrigger>What is LP and how does it work?</AccordionTrigger>
              <AccordionContent>
                <p className="text-sm text-muted-foreground">
                  LP (Liquidity Pool) allows your mSOL and USDC to be deployed to decentralized exchanges like Raydium
                  or Orca. By providing liquidity, you earn additional trading fees on top of your staking rewards. This
                  increases your overall yield but comes with additional risks like impermanent loss.
                </p>
              </AccordionContent>
            </AccordionItem>
          )}

          <AccordionItem value="item-3">
            <AccordionTrigger>What are the risks?</AccordionTrigger>
            <AccordionContent>
              <div className="space-y-4 text-sm">
                <div className="flex gap-2">
                  <AlertTriangle className="h-5 w-5 text-yellow-500 shrink-0 mt-0.5" />
                  <div>
                    <h4 className="font-medium">Smart Contract Risk</h4>
                    <p className="text-muted-foreground">
                      All products are subject to smart contract risks. Our contracts are audited, but no system is
                      completely risk-free.
                    </p>
                  </div>
                </div>

                {(productType === "msol" || productType === "msol-usdc") && (
                  <div className="flex gap-2">
                    <AlertTriangle className="h-5 w-5 text-yellow-500 shrink-0 mt-0.5" />
                    <div>
                      <h4 className="font-medium">Medium Risk Pool</h4>
                      <p className="text-muted-foreground">
                        The Medium Risk pool has additional risk from lending activities, though loans are
                        over-collateralized.
                      </p>
                    </div>
                  </div>
                )}

                {(productType === "msol" || productType === "msol-usdc") && (
                  <div className="flex gap-2">
                    <AlertTriangle className="h-5 w-5 text-red-500 shrink-0 mt-0.5" />
                    <div>
                      <h4 className="font-medium">High Risk Pool</h4>
                      <p className="text-muted-foreground">
                        The High Risk pool has the highest risk due to the inability to withdraw early and exposure to
                        market volatility during the lock period.
                      </p>
                    </div>
                  </div>
                )}

                {productType === "msol-usdc" && (
                  <div className="flex gap-2">
                    <AlertTriangle className="h-5 w-5 text-yellow-500 shrink-0 mt-0.5" />
                    <div>
                      <h4 className="font-medium">LP Risks</h4>
                      <p className="text-muted-foreground">
                        Providing liquidity to pools exposes you to impermanent loss if the price ratio between mSOL and
                        USDC changes significantly.
                      </p>
                    </div>
                  </div>
                )}
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
                If you have any questions about{" "}
                {productType === "msol" ? "mSOL staking" : productType === "usdc" ? "USDC staking" : "dual staking"},
                please refer to our documentation or contact our support team.
              </p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
