const Redis = require("ioredis");

// Connect to Zedis
const redis = new Redis({
  host: "localhost",
  port: 6379,
});

async function main() {
  console.log("--- 01 Basic Key-Value Operations ---");

  // SET
  console.log("Setting key 'app:status' to 'running'...");
  await redis.set("app:status", "running");

  // GET
  const status = await redis.get("app:status");
  console.log(`Got value: ${status}`);

  // EXPIRY
  console.log("Setting key 'session:temp' with 10s expiry...");
  await redis.set("session:temp", "xyz", "EX", 10);
  const ttl = await redis.ttl("session:temp");
  console.log(`TTL: ${ttl}`);

  // PING
  const pong = await redis.ping();
  console.log(`PING response: ${pong}`);

  redis.disconnect();
}

main();
