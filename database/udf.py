from risingwave.udf import udf, UdfServer 
import base64 


@udf(input_types='VARCHAR', result_type='STRUCT<INT, INT>') 
def ttn_payload_decoder(payload): 
    payload_decoded = base64.b64decode(payload) 
    payload_wifi = int.from_bytes( 
        payload_decoded[0:2], byteorder='little', signed=False) 
    payload_ble = int.from_bytes( 
        payload_decoded[2:4], byteorder='little', signed=False) 
    
    return payload_wifi, payload_ble 


if __name__ == '__main__': 
    server = UdfServer(location="0.0.0.0:8815") 
    server.add_function(ttn_payload_decoder) 
    server.serve() 