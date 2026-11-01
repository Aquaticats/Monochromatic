import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider';

//region Seat recording client

/**
 The refusal of a text call, which the scripted client does not serve.

 @example
 ```ts
 await refuseChatText().catch(report);
 ```
 */
function refuseChatText(): Promise<never> {
  return Promise.reject(new Error('chatText unused',),);
}

/**
 The refusal of a quota call, which the scripted client does not serve.

 @example
 ```ts
 await refuseQuotas().catch(report);
 ```
 */
function refuseQuotas(): Promise<never> {
  return Promise.reject(new Error('quotas unused',),);
}

/**
 Client that answers every question with an unreadable reply and keeps the seat each follow-up carried.

 @param seats - list the seats are pushed onto, in the order the calls arrive

 @returns Client whose `chatJson` records `reaskSeat` and whose other calls refuse

 @example
 ```ts
 const client = seatRecordingClient({ seats: [], },);
 ```
 */
export function seatRecordingClient({ seats, }: { readonly seats: ReaskSeat[]; },): ModelClient {
  return {
    chatText: refuseChatText,
    chatJson: function chatJson<ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> {
      if (request.reaskSeat !== undefined)
        seats.push(request.reaskSeat,);
      return Promise.resolve({
        kind: 'schema-mismatch',
        rawText: '',
        detail: 'fixture',
      },);
    },
    quotas: refuseQuotas,
  };
}

//endregion Seat recording client
