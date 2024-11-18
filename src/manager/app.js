import { createClient } from "npm:redis@4.6.4";

// the number of workers
const workerCount = Deno.env.get("WORKER_COUNT");
// connect to redis
const client = createClient({
    url: "redis://redis-streams:6379",
    pingInterval: 1000,
})


try {
    await client.connect();
    console.log("success to connect to redis");
} catch (error) {
    console.error("failure to connect to redis", error);
}

// change into number from id
const stringToNumber = (str) => {
    let sum = 0;
    for (let i = 0; i < str.length; i++) {
      sum += str.charCodeAt(i);
    }
    return sum;
};

// make to stream name
const streamName = (str) => {
    let id_number = stringToNumber(str) % workerCount + 1;
    return `s${id_number}`;
};

while(1){
    let stock_data = await client.xReadGroup(
        'managers',
        "manager1",
      {
        key: 'ingress',
        id: '>'
      }, {
        count: 1,
        block: 0
    });
    if(stock_data){
        const message = stock_data[0].messages[0].message;
        
        let id = message.id;
        let sectype = message.sectype;
        let last = message.last;
        let time = message.time;
        let date = message.date;
        let queue_name = streamName(id);
        try {
            await client.xAdd(queue_name, '*',{
            id: id,
            sectype: sectype,
            last: last,
            time: time,
            date: date
        });
        } catch (error) {
            console.error('failiure to  scale', error);
        }
        stock_data = null;
    }
}
