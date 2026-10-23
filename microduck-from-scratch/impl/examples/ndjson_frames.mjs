// NDJSON：换行就是帧边界。模拟一条连接上分两片到达的数据。
let buf = "";
const chunks = ['{"id":1,"method":"hello"}\n{"id":2,"met', 'hod":"health"}\n'];

for (const chunk of chunks) {
  buf += chunk;
  const lines = buf.split("\n");
  buf = lines.pop(); // 最后一段可能不完整，留到下一片到了再拼
  for (const line of lines) console.log("完整帧:", JSON.parse(line));
}
console.log("缓冲区残留:", JSON.stringify(buf));
