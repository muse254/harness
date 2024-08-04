export const idlFactory = ({ IDL }) => {
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const TransformArgs = IDL.Record({
    'context' : IDL.Vec(IDL.Nat8),
    'response' : HttpResponse,
  });
  const HarnessResult = IDL.Record({
    'data' : IDL.Opt(IDL.Text),
    'error' : IDL.Text,
    'success' : IDL.Bool,
  });
  return IDL.Service({
    'get_devices' : IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
    'get_program_code' : IDL.Func([], [IDL.Vec(IDL.Nat8)], ['query']),
    'harness_transform' : IDL.Func([TransformArgs], [HttpResponse], ['query']),
    'hello' : IDL.Func([IDL.Text], [HarnessResult], []),
    'register_device' : IDL.Func([IDL.Text], [], []),
    'remove_device' : IDL.Func([IDL.Text], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
