import { serve } from "./deps.js";
import { createClient } from "npm:redis@4.6.4";

// the number of workers
const workerCount = Deno.env.get("WORKER_COUNT");
// connect to redis
const client = createClient({
    url: "redis://redis:6379",
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

// function of url map
const test = async (request) => {
    let id = "id3";
    let sectype = 'I';
    let last = '11.105000';
    let time = '09:19:44:743';
    let date = '08-11-2021';
    let queue_name = streamName(id);
    try {
      await client.xAdd(queue_name, '*',{
        id: id,
        sectype: sectype,
        last: last,
        time: time,
        date: date
    });
      return new Response("OK", { status: 200 }); 
    } catch (error) {
      console.error('failiure to  scale', error);
      return new Response("Error", { status: 500 });
    }
  };

// url mapping
const urlMapping = [
    {
      method: "GET",
      pattern: new URLPattern({ pathname: "/" }),
      fn: test,
    }
];

// server handling
const handleRequest = async (request) => {
    const mapping = urlMapping.find(
      (um) => um.method === request.method && um.pattern.test(request.url)
    );
  
    if (!mapping) {
      return new Response("Not found", { status: 404 });
    }
  
    const mappingResult = mapping.pattern.exec(request.url);
    try {
      return await mapping.fn(request, mappingResult);
    } catch (e) {
      console.log(e);
      return new Response(e.stack, { status: 500 })
    }
};
  
const portConfig = { port: 7777, hostname: "0.0.0.0" };
serve(handleRequest, portConfig);
  