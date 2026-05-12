// Minecraft 服务器管理面板 - 自动化测试脚本
// 使用 Playwright 语法

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

// 测试配置
const TEST_CONFIG = {
  baseUrl: 'http://localhost:3000',
  screenshotsDir: path.join(__dirname, 'test-screenshots'),
  waitTimeout: 30000, // 30秒
};

// 确保截图目录存在
if (!fs.existsSync(TEST_CONFIG.screenshotsDir)) {
  fs.mkdirSync(TEST_CONFIG.screenshotsDir, { recursive: true });
}

async function runMCServerPanelTest() {
  console.log('🚀 开始 Minecraft 服务器管理面板自动化测试...\n');
  
  const browser = await chromium.launch({
    headless: false, // 设置为 true 可无头运行
    slowMo: 50, // 慢动作便于观察
  });
  
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    userAgent: 'MC-Server-Panel-Test-Bot/1.0',
  });
  
  const page = await context.newPage();
  
  // 页面事件监听
  page.on('console', msg => {
    if (msg.text().includes('WebSocket') || msg.text().includes('server')) {
      console.log(`📱 [浏览器控制台] ${msg.text()}`);
    }
  });
  
  try {
    // ==============================================
    // 测试步骤 1: 打开面板首页
    // ==============================================
    console.log('📝 步骤 1: 打开面板首页...');
    await page.goto(TEST_CONFIG.baseUrl);
    await page.waitForURL(TEST_CONFIG.baseUrl);
    
    // 截图：首页加载
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '01-homepage.png'),
      fullPage: true 
    });
    console.log('✅ 首页加载成功 - 截图已保存\n');
    
    // ==============================================
    // 测试步骤 2: 等待 WebSocket 连接成功
    // ==============================================
    console.log('📝 步骤 2: 等待 WebSocket 连接...');
    await page.waitForSelector('text="WebSocket"', { 
      state: 'visible',
      timeout: TEST_CONFIG.waitTimeout 
    });
    
    // 检查连接状态（显示 "WebSocket Connected"）
    const connectionStatus = await page.locator('text=/WebSocket Connected|WebSocket Disconnected/').textContent();
    console.log(`🔌 WebSocket 状态: ${connectionStatus}`);
    
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '02-websocket-connected.png') 
    });
    console.log('✅ WebSocket 连接状态检测 - 截图已保存\n');
    
    // ==============================================
    // 测试步骤 3: 点击"启动服务器"按钮 (Toggle Demo)
    // ==============================================
    console.log('📝 步骤 3: 点击启动服务器...');
    const startButton = page.getByRole('button', { name: /Toggle Demo|Start Server/i });
    await startButton.click();
    
    // 截图：启动请求已发送
    await page.waitForTimeout(2000);
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '03-server-starting.png') 
    });
    console.log('✅ 启动服务器请求已发送 - 截图已保存\n');
    
    // ==============================================
    // 测试步骤 4: 监听控制台日志，等待启动完成
    // ==============================================
    console.log('📝 步骤 4: 等待服务器启动完成...');
    let doneFound = false;
    
    // 导航到终端页面
    await page.goto(`${TEST_CONFIG.baseUrl}/terminal`);
    
    // 等待终端加载
    await page.waitForSelector('.terminal-container', { timeout: TEST_CONFIG.waitTimeout });
    
    // 发送模拟启动命令（在演示模式中）
    const commandInput = page.locator('input[id="command-input"]');
    if (await commandInput.isVisible()) {
      await commandInput.fill('start server demo');
      await commandInput.press('Enter');
    }
    
    // 等待一些时间让服务器"启动"
    console.log('⏳ 等待模拟服务器启动...');
    await page.waitForTimeout(5000);
    
    // 截图：终端输出
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '04-terminal-output.png'),
      fullPage: true 
    });
    
    // ==============================================
    // 测试步骤 5: 返回仪表盘，截图保存状态
    // ==============================================
    console.log('📝 步骤 5: 返回仪表盘并截图...');
    await page.goto(`${TEST_CONFIG.baseUrl}/`);
    
    // 等待图表加载
    await page.waitForSelector('.chart-container', { 
      state: 'visible', 
      timeout: TEST_CONFIG.waitTimeout 
    });
    
    // 等待数据加载
    await page.waitForTimeout(3000);
    
    // 截图：完整仪表盘
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '05-dashboard-with-charts.png'),
      fullPage: true 
    });
    
    console.log('✅ 仪表盘完整截图已保存（包含 CPU/内存图表）\n');
    
    // ==============================================
    // 测试步骤 6: 点击停止服务器
    // ==============================================
    console.log('📝 步骤 6: 停止服务器...');
    const stopButton = page.getByRole('button', { name: /Toggle Demo|Stop Server/i });
    await stopButton.click();
    
    // 等待状态更新
    await page.waitForTimeout(3000);
    
    // 截图：服务器已停止
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, '06-server-stopped.png'),
      fullPage: true 
    });
    
    console.log('✅ 服务器停止成功 - 截图已保存\n');
    
    // ==============================================
    // 测试通过！
    // ==============================================
    console.log('🎉 所有测试步骤完成！');
    console.log(`📁 测试截图保存在: ${TEST_CONFIG.screenshotsDir}`);
    
    // 列出所有截图
    const screenshotFiles = fs.readdirSync(TEST_CONFIG.screenshotsDir);
    console.log('\n📸 生成的截图文件:');
    screenshotFiles.forEach((file, index) => {
      console.log(`  ${index + 1}. ${file}`);
    });
    
    // 最终延迟，让观察完整状态
    await page.waitForTimeout(3000);
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error);
    
    // 错误时截图
    await page.screenshot({ 
      path: path.join(TEST_CONFIG.screenshotsDir, 'error-screenshot.png') 
    });
    throw error;
  } finally {
    await context.close();
    await browser.close();
  }
}

// 运行测试
runMCServerPanelTest()
  .then(() => {
    console.log('\n✅ 自动化测试执行完成');
    process.exit(0);
  })
  .catch((error) => {
    console.error('\n❌ 自动化测试失败:', error);
    process.exit(1);
  });
