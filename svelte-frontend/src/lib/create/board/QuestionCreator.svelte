<script lang="ts">
  import type { Media, MediaType, NumberScope, Question, QuestionType, VideoType } from 'cult-common';
  import { JeopardyBoardCreatorStore } from './BoardCreatorsStore';
  import { match } from 'ts-pattern';

  export let question: Question;
  export let index: number;
  export let qIndex: number;

  let selection : Map<number, VideoType> = new Map();

  function update(event: Event) {
      const input = event.target as HTMLInputElement;
      const field = input.dataset.field as keyof Question;
      if (field in question) {
          question[field] = input.value;
          JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
      }
  }

  function handleQuestionTypeChange(event: Event) {
      const select = event.target as HTMLSelectElement;
      const value = select.value
      
      question.question_type = match(value)
          .with('Question', () => "Question" as QuestionType)
          .with('Media', () => ({ Media: [] }))
          .with('Youtube', () => ({ Youtube: "" }))
          .otherwise(() => question.question_type);

      JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
  }

  function updateMediaName(event: Event, index:number) {
      const input = event.target as HTMLInputElement;
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
          question.question_type.Media[index].name = input.value;
      }
      JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
      
  }

  function addMedia() {
      console.log("ADD MEDIA"); 
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
        
        let media : Media = {
          media_type : "Image",
          name: "test.png",
        };
        question.question_type.Media.push(media);
        console.log(question);

        JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
      }
  }

  function updateMediaType(event: Event, media_index:number) {
      const select = event.target as HTMLSelectElement;
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
          let type = match(select.value)
              .with('Image', () => "Image" as MediaType)
              .with('Audio', () => "Audio" as MediaType)
              .with('Text', () => "Text" as MediaType)
              .with('Pdf', () => "Pdf" as MediaType)
              .with('Video', () => ({ Video: [] }) as MediaType)
              .otherwise(() => "Unknown" as MediaType);
          question.question_type.Media[media_index].media_type = type;
      }
      JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
  }

  function changeVideoType(id:number, media_index:number)  {
      console.log("ADD VIDEO TYPE");
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
        if (typeof question.question_type.Media[media_index].media_type === "object" &&  "Video" in  question.question_type.Media[media_index].media_type) {
          let videoTypes : VideoType[]= [];
          console.log("BEFORE", selection, id);
          //contains id remove it
          if (selection.has(id)) {
            selection.delete(id);
          } else {
            let type = match(id)
              .with(1, () => "Mute" as VideoType)
              .with(2, () => ({ TimeSlots: [{start: 0, end:10}] }) as VideoType)
              .with(3, () => ({ Slowmotion: 100 }) as VideoType)
              .otherwise(() => "None" as VideoType);
            selection.set(id, type);
          }
          

          for (let selection2 of selection.values()) {
            videoTypes.push(selection2);
          }

          console.log("AFTER", selection, videoTypes)
          question.question_type.Media[media_index].media_type.Video = videoTypes;
          console.log("AFTER!!", question.question_type.Media[media_index].media_type.Video );
          JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
        }
    }
  }   


  function updateSlowmotion(e, media_index:number)  {
    console.log("updateVideoType VIDEO TYPE");
    let value = e.target.value;
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
        let media = question.question_type.Media[media_index];
        if (typeof media.media_type === "object" &&  "Video" in media.media_type) {
              if ("Slowmotion" in media.media_type.Video) {
                media.media_type.Video.Slowmotion = value;
                JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
          }
        }
      }
  }

  function updateTimeSlotStart(slot_imdex: number, e, media_index:number)  {
    console.log("updateVideoType VIDEO TYPE");
    let value = e.target.value;
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
        let media = question.question_type.Media[media_index];
        if (typeof media.media_type === "object" &&  "Video" in media.media_type) {
              if ("TimeSlots" in media.media_type.Video) {
                let slots = media.media_type.Video.TimeSlots as NumberScope[];
                slots[slot_imdex].start = value;
                media.media_type.Video.TimeSlots = slots;
                JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
          }
        }
      }
  }

  function updateTimeSlotEnd(slot_imdex: number, e, media_index:number)  {
    let value = e.target.value;
    console.log("updateVideoType VIDEO TYPE");
      if (typeof question.question_type === "object" && "Media" in question.question_type) {
        let media = question.question_type.Media[media_index];
        if (typeof media.media_type === "object" &&  "Video" in media.media_type) {
              if ("TimeSlots" in media.media_type.Video) {
                let slots = media.media_type.Video.TimeSlots as NumberScope[];
                slots[slot_imdex].start = value;
                media.media_type.Video.TimeSlots = slots;
                JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
          }
        }
      }
  }

  function addTimeSlot(media_index:number, video_id:number) {
    console.log("ADD TIME SLOT");
    if (typeof question.question_type === "object" && "Media" in question.question_type) {
      let media = question.question_type.Media[media_index];
      if (typeof media.media_type === "object" &&  "Video" in media.media_type) {
          if (selection.has(2)) {
            let slots = selection.get(2);
            if (slots&& typeof slots == "object" && "TimeSlots" in slots) {
              slots.TimeSlots.push({ start: 0, end: 0 });
              selection.set(2, slots);
              let videos : VideoType[] = [];
              for (let i = 0; i < selection.size; i++) {
                let type = selection.get(i);
                if (type)
                  videos.push(type);
              }
              media.media_type.Video = videos;
              JeopardyBoardCreatorStore.setQuestion(index, qIndex, question);
          }
        }
      }
    }
  }
</script>

<div class="p-2 border-b">
  <select on:change={handleQuestionTypeChange} value={question.question_type as string}>
      <option value="Question">Question</option>
      <option value="Media">Media</option>
      <option value="Youtube">Youtube</option>
  </select>
  {#if typeof question.question_type === "string"}
    {#if question.question_type === "Question"}
      <input type="text"  data-field="question" on:input={update} value={question.question} placeholder="Enter Question" class="border px-2 py-1 rounded w-full"/>
      <input  type="number" data-field="value" value={question.value} placeholder="Value"  class="border px-2 py-1 rounded w-full mt-2"/>
      <input type="text"  data-field="answer" on:input={update} value={question.answer} placeholder="Answer" class="border px-2 py-1 rounded w-full mt-2"/>
    {/if}
  {:else}
    {#if "Media" in question.question_type}
        <button on:click={addMedia} class="bg-green-500 text-white px-4 py-2 rounded">Add Media</button>
    
      {#each question.question_type.Media as media, media_index}
        <input type="text"data-field="name"on:input={(e) => updateMediaName(e, media_index)}  value={media.name} placeholder="Media Name"class="border px-2 py-1 rounded w-full mt-2"/>
        <select data-field="media_type" on:input={(e) => updateMediaType(e, media_index)} value={media.media_type}>
          <option value="Image">Image</option>
          <option value="Audio">Audio</option>
          <option value="Text">Text</option>
          <option value="Pdf">Pdf</option>
          <option value="Video">Video</option>
        </select>
        {typeof media.media_type === "object"}
        {#if typeof media.media_type === "object"}
          {#if "Video" in media.media_type}
            <div class="flex flex-col">
              <label><input type="checkbox" on:change={(e) => changeVideoType(1, media_index)}/> Mute</label>
              <label><input type="checkbox" on:change={(e) => changeVideoType(2, media_index)}/> TimeSlots</label>
              <label><input type="checkbox" on:change={(e) => changeVideoType(3, media_index)}/> Slowmotion</label>
              {#each media.media_type.Video as videotype, video_id}
                {#if typeof videotype === "object"}
                  {#if "TimeSlots" in videotype}
                      <button on:click={() => addTimeSlot(media_index, video_id)}>Add Time Slot</button>
                      <h3>TimeSlots:</h3>
                      {#each videotype.TimeSlots as slot, slotIndex}
                        <div class="flex"> {slotIndex + 1}:
                          Start: <input type="start" on:input={(e) => updateTimeSlotStart(slotIndex, e, media_index)} value={slot.end} placeholder="TimeSlot"/>
                          End: <input type="end" on:input={(e) => updateTimeSlotEnd(slotIndex, e, media_index)} value={slot.end} placeholder="TimeSlot"/>
                        </div>
                      {/each}
                  {/if}
                  {#if "Slowmotion" in videotype}
                    <h3>Slowmotion:</h3>
                    <input type="number" on:input={(e) => updateSlowmotion(e,media_index)} value={videotype.Slowmotion} placeholder="Slowmotion"/>
                  {/if}
                {/if}
              {/each}
            </div>
          {/if}
        {/if}
      {/each}
    {:else if "Youtube" in question.question_type}
      <input type="text" data-field="Youtube" on:input={update} value={question.question_type.Youtube} placeholder="Youtube ID" class="border px-2 py-1 rounded w-full"/>
    {/if}
  {/if}
</div>

