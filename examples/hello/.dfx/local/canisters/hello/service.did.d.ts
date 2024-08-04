import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface HarnessResult {
  'data' : [] | [string],
  'error' : string,
  'success' : boolean,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponse,
}
export interface _SERVICE {
  'get_devices' : ActorMethod<[], Array<string>>,
  'get_program_code' : ActorMethod<[], Uint8Array | number[]>,
  'harness_transform' : ActorMethod<[TransformArgs], HttpResponse>,
  'hello' : ActorMethod<[string], HarnessResult>,
  'register_device' : ActorMethod<[string], undefined>,
  'remove_device' : ActorMethod<[string], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
