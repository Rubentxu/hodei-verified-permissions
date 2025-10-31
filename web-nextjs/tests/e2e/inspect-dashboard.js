const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  await page.goto('http://localhost:3002/dashboard');
  await page.waitForLoadState('networkidle');

  // Get the full HTML structure
  const html = await page.content();
  console.log('=== HTML COMPLETO ===');
  console.log(html.substring(0, 5000));

  // Get all text content
  const textContent = await page.evaluate(() => document.body.innerText);
  console.log('\n=== TEXTO VISIBLE ===');
  console.log(textContent.substring(0, 2000));

  // Get specific elements
  console.log('\n=== ELEMENTOS ESPECÍFICOS ===');
  const h2Count = await page.locator('h2').count();
  console.log(`Cantidad de h2: ${h2Count}`);
  for (let i = 0; i < h2Count; i++) {
    const text = await page.locator('h2').nth(i).textContent();
    console.log(`  h2[${i}]: ${text}`);
  }

  // Check for specific text
  console.log('\n=== BÚSQUEDAS ESPECÍFICAS ===');
  const hasSystemHealth = await page.locator('text=System Health').count();
  console.log(`"System Health" aparece: ${hasSystemHealth} veces`);

  const hasPolicyStores = await page.locator('text=Policy Stores').count();
  console.log(`"Policy Stores" aparece: ${hasPolicyStores} veces`);

  const hasGrpcServer = await page.locator('text=gRPC Server').count();
  console.log(`"gRPC Server" aparece: ${hasGrpcServer} veces`);

  await browser.close();
})();